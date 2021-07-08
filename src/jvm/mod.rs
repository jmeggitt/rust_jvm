use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::io;
use std::io::{Error, ErrorKind};
use std::mem::replace;
use std::option::Option::Some;
use std::path::PathBuf;
use std::rc::Rc;

use hashbrown::{HashMap, HashSet};
use jni::sys::{jobject, jvalue};
use libloading::Library;
use walkdir::WalkDir;

pub use mem::*;

use crate::attribute::CodeAttribute;
use crate::class::{AccessFlags, BufferedRead, Class, ClassLoader, MethodInfo};
use crate::constant_pool::Constant;
use crate::jvm::hooks::register_hooks;
use crate::jvm::mem_rewrite::ClassSchema;
use crate::jvm::stack::OperandStack;
use crate::types::FieldDescriptor;
use std::sync::Arc;

macro_rules! fatal_error {
    ($($arg:tt),*) => {{
        error!($($arg),*);
        panic!($($arg),*);
    }};
}

// This was generated from jni.h so allow any variations
// #[allow(warnings)]
// pub mod bindings;

mod mem;

mod hooks;
mod interface;
mod stack;

#[cfg(unix)]
mod exec;
mod internals;
mod mem_rewrite;

pub struct StackFrame {
    // Either an object or class
    pub target: Rc<UnsafeCell<Object>>,
    // Comparable to the .text section of a binary
    pub constants: Vec<Constant>,
    // Values treated as registers
    pub locals: Vec<LocalVariable>,
    // The stack frame
    pub stack: Vec<LocalVariable>,
    // Instruction pointer
    // pub rip: usize,
    pub branch_offset: i64,
    // Instructions within function
    // pub code: CodeAttribute,
    // Work around so instructions can set the return value
    pub returns: Option<Option<LocalVariable>>,
    // Kinda like a sticky fault
    pub throws: Option<LocalVariable>,
}

impl StackFrame {
    pub fn new(
        target: Rc<UnsafeCell<Object>>,
        max_locals: usize,
        max_stack: usize,
        constants: Vec<Constant>,
        args: Vec<LocalVariable>,
    ) -> Self {
        let mut locals = vec![LocalVariable::Int(0); max_locals];
        for (idx, value) in args.into_iter().enumerate() {
            locals[idx] = value;
        }

        StackFrame {
            target,
            constants,
            locals,
            stack: Vec::with_capacity(max_stack),
            // rip: 0,
            branch_offset: 0,
            returns: None,
            throws: None,
        }
    }

    pub fn debug_print(&self) {
        debug!("Stack Frame Debug:");
        debug!("\tTarget: {:?}", &self.target);
        debug!("\tBranching Offset: {:?}", self.branch_offset);
        // debug!("\tInstruction Pointer: {:?}", self.rip);
        debug!("\tReturn Slot: {:?}", &self.returns);

        debug!("\tLocal Variables: {}", self.locals.len());
        for (idx, local) in self.locals.iter().enumerate() {
            debug!("\t\t{}:\t{:?}", idx, local)
        }

        debug!(
            "\tOperand Stack: {}/{}",
            self.stack.len(),
            self.stack.capacity()
        );
        for (idx, local) in self.stack.iter().enumerate() {
            debug!("\t\t{}:\t{:?}", idx, local)
        }
    }

    pub fn exec(
        &mut self,
        jvm: &mut JVM,
        code: &CodeAttribute,
    ) -> Result<Option<LocalVariable>, LocalVariable> {
        // let instructions = self.code.instructions.clone();
        for (offset, instruction) in &code.instructions {
            trace!("\t{}:\t{:?}", offset, instruction);
        }

        let mut rip = 0;
        loop {
            if rip >= code.instructions.len() {
                // panic!("Reached function end without returning");
                return Ok(None);
            }

            // let instruction = &self.code.instructions[self.rip];
            debug!("Executing instruction {:?}", &code.instructions[rip]);
            code.instructions[rip].1.exec(self, jvm);

            if let Some(v) = &self.returns {
                return Ok(v.clone());
            }

            if self.throws.is_some() {
                let exception = replace(&mut self.throws, None).unwrap();

                // Determine exception type
                let exception_class = match &exception {
                    LocalVariable::Reference(Some(v)) => unsafe { (&*v.get()).expect_class() },
                    _ => panic!("Unable to get class of exception"),
                }
                .unwrap();

                // Figure out if can be caught or if it needs to propagate
                let position = code.instructions[rip].0;
                match code.attempt_catch(position, &exception_class, &self.constants, jvm) {
                    Some(jump_dst) => {
                        debug!("Exception successfully caught, branching to catch block!");
                        self.branch_offset = jump_dst as i64 - position as i64;
                    }
                    None => {
                        warn!(
                            "Raised exception from instruction {:?}",
                            &code.instructions[rip]
                        );
                        warn!("Exception not caught, Raising: {}", exception_class);
                        jvm.debug_print_call_stack();
                        return Err(exception);
                    }
                }
            }

            if self.branch_offset == 0 {
                rip += 1;
            }

            // if self.branch_offset == 0 && rip == code.instructions.len() {
            //     return None;
            // }

            while self.branch_offset != 0 {
                debug!("Branch offset is {} and rip is {}", self.branch_offset, rip);
                debug!(
                    "Currently pointed at instruction {:?}",
                    &code.instructions[rip]
                );
                let (current_pos, _) = code.instructions[rip];
                if self.branch_offset > 0 {
                    rip += 1;
                } else {
                    rip -= 1;
                }
                let (new_pos, _) = code.instructions[rip];
                let offset = new_pos as i64 - current_pos as i64;

                // if (self.branch_offset >= 0) != (self.branch_offset - offset >= 0) {
                //     panic!("Branch skip did not skip to valid op code!");
                // }

                self.branch_offset -= offset;
            }
        }
    }
}

pub struct JVM {
    pub class_loader: ClassLoader,
    pub static_fields: HashMap<String, LocalVariable>,

    // Classes we have loaded and called <clinit> if possible
    pub static_load: HashSet<String>,
    pub native_stack: OperandStack,
    pub linked_libraries: NativeManager,
    pub registered_classes: HashMap<String, Rc<UnsafeCell<Object>>>,

    pub call_stack: Vec<(Rc<UnsafeCell<Object>>, String)>,

    schemas: HashMap<String, Arc<ClassSchema>>,
}

impl JVM {
    pub fn new(class_loader: ClassLoader) -> Self {
        let mut jvm = JVM {
            class_loader,
            static_fields: HashMap::new(),
            static_load: HashSet::new(),
            native_stack: OperandStack::new(16384),
            linked_libraries: NativeManager::new(),
            // locals: vec![LocalVariable::Int(0); 255],
            registered_classes: HashMap::new(),
            call_stack: Vec::new(),
            schemas: HashMap::new(),
        };

        unsafe {
            jvm.make_primary_jvm();
        }

        // Tell jvm to skip initializing the security manager because it requires a ton of native methods.
        // jvm.static_load.insert("java/lang/SecurityManager".to_string());

        // warn!("Loading core")
        jvm.load_core_libs().unwrap();
        register_hooks(&mut jvm);

        jvm
    }

    pub fn class_schema(&mut self, class: &str) -> Option<Arc<ClassSchema>> {
        if !self.schemas.contains_key(class) {
            let class_spec = self.expect_class(class).clone();

            let schema = ClassSchema::build(&class_spec, self);
            self.schemas.insert(class.to_string(), Arc::new(schema));
        }

        self.schemas.get(class).cloned()
    }

    pub fn get_class_instance(&mut self, name: &str) -> Rc<UnsafeCell<Object>> {
        if let Some(class) = self.registered_classes.get(name) {
            return class.clone();
        }

        let class = Object::build_class(self, name);
        self.registered_classes
            .insert(name.to_string(), class.clone());
        class
    }

    pub fn instanceof(&self, instance: &str, target: &str) -> Option<bool> {
        // If this is a regular object, we hit the base case
        if instance == "java/lang/Object" {
            return Some(false);
        }

        let entry_class = self.class_loader.class(instance)?;
        let super_class = entry_class.super_class();

        if super_class == target {
            return Some(true);
        }

        for interface in entry_class.interfaces() {
            if interface == target || self.instanceof(&interface, target) == Some(true) {
                return Some(true);
            }
        }

        self.instanceof(&super_class, target)
    }

    pub fn find_instance_method(
        &self,
        class: &str,
        method: &str,
        desc: &str,
    ) -> Option<(String, MethodInfo, Vec<Constant>)> {
        let entry_class = self.class_loader.class(class)?;

        if let Some(main_method) = entry_class.get_method(method, desc) {
            return Some((
                class.to_string(),
                main_method.clone(),
                entry_class.constants().to_vec(),
            ));
        }

        if class != "java/lang/Object" {
            return self.find_instance_method(&entry_class.super_class(), method, desc);
        }

        None
    }

    pub fn load_core_libs(&mut self) -> io::Result<()> {
        let lib_dir = self.class_loader.class_path().java_home().join("lib");
        info!("Loading shared libraries from {}", lib_dir.display());

        // We need to load this first since the following libraries depend on it
        #[cfg(unix)]
        self.linked_libraries
            .load_library(lib_dir.join("amd64/server/libjvm.so"))?;
        #[cfg(windows)]
        self.linked_libraries
            .load_library(lib_dir.join("bin/server/jvm.dll"))?;

        // Load includes in deterministic order to ensure regularity between runs
        for entry in WalkDir::new(&lib_dir).sort_by_file_name() {
            let entry = entry?;
            if !entry.path().is_file() {
                continue;
            }

            #[cfg(unix)]
            if entry.path().extension() == Some("so".as_ref()) {
                self.linked_libraries
                    .load_library(entry.path().to_path_buf())?;
            }

            #[cfg(windows)]
            if entry.path().extension() == Some("dll".as_ref()) {
                self.linked_libraries
                    .load_library(entry.path().to_path_buf())?;
            }
        }

        Ok(())
    }

    pub unsafe fn make_primary_jvm(&mut self) {
        let ptr = self as *mut Self;
        interface::GLOBAL_JVM = Some(Box::from_raw(ptr));
    }

    pub fn build_string(&mut self, string: &str) -> LocalVariable {
        let char_array =
            LocalVariable::Reference(Some(Rc::new(UnsafeCell::new(Object::from(string)))));
        // self.init_class("java/lang/String");
        self.class_loader.attempt_load("java/lang/String").unwrap();

        let args = vec![char_array];
        self.exec_static(
            "java/lang/String",
            "valueOf",
            "([C)Ljava/lang/String;",
            args,
        )
        .unwrap()
        .unwrap()
    }

    fn expect_class(&mut self, class: &str) -> &Class {
        self.class_loader.class(class).unwrap()
    }

    pub fn debug_print_call_stack(&self) {
        debug!("Call stack:");
        let mut padding = String::new();
        for debug_str in &self.call_stack {
            debug!("{}{}", &padding, debug_str.1);
            padding.push_str("   ");
        }
    }

    pub fn exec(
        &mut self,
        target: Rc<UnsafeCell<Object>>,
        class_name: &str,
        method: MethodInfo,
        constants: Vec<Constant>,
        mut args: Vec<LocalVariable>,
    ) -> Result<Option<LocalVariable>, LocalVariable> {
        let call_string = format!(
            "{}::{} {}",
            class_name,
            method.name(&constants).unwrap(),
            method.descriptor(&constants).unwrap()
        );
        debug!("Executing method {} for target {:?}", &call_string, &target);

        let target_class = self.get_class_instance(class_name);
        self.call_stack.push((target_class, call_string));
        self.debug_print_call_stack();

        let ret = if method.access.contains(AccessFlags::NATIVE) {
            let fn_ptr = match self.linked_libraries.get_fn_ptr(
                class_name,
                &method.name(&constants).unwrap(),
                &method.descriptor(&constants).unwrap(),
            ) {
                Some(v) => v,
                None => panic!(
                    "Unable to find function ptr {}::{} {}",
                    class_name,
                    &method.name(&constants).unwrap(),
                    &method.descriptor(&constants).unwrap()
                ),
            };

            if let Ok(FieldDescriptor::Method { returns, .. }) =
                FieldDescriptor::read_str(&method.descriptor(&constants).unwrap())
            {
                let ret = unsafe {
                    debug!("Native method arguments:");
                    let raw_args = args
                        .iter_mut()
                        .map(|x| {
                            debug!("\t{:?}", x);
                            match x {
                                LocalVariable::Reference(Some(v)) => jvalue {
                                    l: v as *mut _ as jobject,
                                },
                                x => {
                                    let value: jvalue = x.clone().into();
                                    value
                                }
                            }
                        })
                        .collect();

                    self.native_stack.native_method_call(
                        fn_ptr,
                        &target as *const _ as jobject,
                        raw_args,
                    )
                };

                Ok(returns.cast(ret))
            } else {
                panic!("Method descriptor can not be correctly parsed")
            }
        } else {
            if let Object::Instance { .. } = unsafe { &*target.get() } {
                args.insert(0, LocalVariable::Reference(Some(target.clone())));
            }
            let code = method.code(&constants);
            let mut frame = StackFrame::new(
                target,
                code.max_locals as usize,
                code.max_stack as usize,
                constants,
                args,
            );
            frame.exec(self, &code)
        };

        self.call_stack.pop();
        ret
    }

    pub fn exec_method(
        &mut self,
        target: Rc<UnsafeCell<Object>>,
        method: &str,
        desc: &str,
        args: Vec<LocalVariable>,
    ) -> Result<Option<LocalVariable>, LocalVariable> {
        let lock = unsafe { &*target.get() };
        let class = lock.expect_class().unwrap();

        let (class_name, main_method, constants) =
            match self.find_instance_method(&class, method, desc) {
                Some(v) => v,
                _ => fatal_error!("Unable to find {}::{} {}", class, method, desc),
            };

        self.exec(target, &class_name, main_method, constants, args)
    }

    pub fn exec_static(
        &mut self,
        class: &str,
        method: &str,
        desc: &str,
        args: Vec<LocalVariable>,
    ) -> Result<Option<LocalVariable>, LocalVariable> {
        let target = Rc::new(UnsafeCell::new(Object::Class(class.to_string())));
        self.exec_method(target, method, desc, args)
    }

    // pub fn preload_class(&mut self, class: &str) {
    //     if !self.static_load.contains(class) {
    //         self.class_loader.attempt_load(class).unwrap();
    //         self.static_load.insert(class.to_string());
    //
    //         if class != "java/lang/Object" {
    //             let super_class = self.class_loader.class(class).unwrap().super_class();
    //             self.preload_class(&super_class);
    //         }
    //     }
    // }

    pub fn init_class(&mut self, class: &str) {
        if !self.static_load.contains(class) {
            self.class_loader.attempt_load(class).unwrap();
            self.static_load.insert(class.to_string());

            if class != "java/lang/Object" {
                let super_class = self.class_loader.class(class).unwrap().super_class();
                self.init_class(&super_class);
            }

            let instance = self.class_loader.class(class).unwrap();
            if instance.get_method("<clinit>", "()V").is_some() {
                self.exec_static(class, "<clinit>", "()V", vec![]).unwrap();
            }
        }
    }

    pub fn entry_point(&mut self, class: &str, args: Vec<String>) -> io::Result<()> {
        info!(
            "Starting entry point class {} with arguments {:?}",
            class, &args
        );

        self.class_loader.attempt_load(class)?;

        // This really should be an instance of a 0 length array
        // LocalVariable::Reference(None));
        let arg = LocalVariable::Reference(Some(Rc::new(UnsafeCell::new(Object::Array {
            values: vec![],
            element_type: FieldDescriptor::Object("java/lang/String".to_string()),
        }))));

        self.exec_static(class, "main", "([Ljava/lang/String;)V", vec![arg])
            .unwrap();

        Ok(())
    }
}

pub fn clean_str(str: &str) -> String {
    let mut out = String::new();
    for c in str.chars() {
        match c {
            '_' => out.push_str("_1"),
            ';' => out.push_str("_2"),
            '[' => out.push_str("_3"),
            '/' => out.push('_'),
            'a'..='z' | 'A'..='Z' | '0'..='9' => out.push(c),
            x => out.push_str(&format!("_{:04x}", x as u32)),
        }
    }
    out
}

#[derive(Default)]
pub struct NativeManager {
    libs: HashMap<PathBuf, Library>,
    load_order: Vec<PathBuf>,
    loaded_fns: HashMap<String, *const c_void>,
}

impl NativeManager {
    pub fn new() -> Self {
        let mut manager = NativeManager::default();
        internals::register_natives(&mut manager);
        manager
    }

    pub fn load_library(&mut self, path: PathBuf) -> io::Result<()> {
        info!("Loading dynamic library {}", path.display());
        if !self.libs.contains_key(&path) {
            unsafe {
                match Library::new(&path) {
                    Ok(v) => {
                        self.load_order.push(path.clone());
                        self.libs.insert(path, v)
                    }
                    Err(e) => return Err(Error::new(ErrorKind::Other, e)),
                };
            }
        }

        Ok(())
    }

    fn clean_desc(x: &str) -> Option<String> {
        Some(clean_str(&x[1..x.find(')')?]))
    }

    pub fn register_fn(
        &mut self,
        class: &str,
        name: &str,
        desc: &str,
        fn_ptr: *const c_void,
    ) -> bool {
        debug!(
            "Registering native function for {}::{} {}",
            class, name, desc
        );
        let long_name = format!(
            "Java_{}_{}__{}",
            clean_str(class),
            clean_str(name),
            Self::clean_desc(desc).unwrap()
        );

        if self.loaded_fns.contains_key(&long_name) {
            error!(
                "Failed to register native function! Already registered: {}",
                long_name
            );
            return false;
        }

        self.loaded_fns.insert(long_name, fn_ptr).is_none()
    }

    pub fn get_fn_ptr(&mut self, class: &str, name: &str, desc: &str) -> Option<*const c_void> {
        let long_name = format!(
            "Java_{}_{}__{}",
            clean_str(class),
            clean_str(name),
            Self::clean_desc(desc)?
        );
        let short_name = format!("Java_{}_{}", clean_str(class), clean_str(name));

        if let Some(v) = self.loaded_fns.get(&long_name) {
            return Some(*v);
        }

        if let Some(v) = self.loaded_fns.get(&short_name) {
            return Some(*v);
        }

        for lib_path in &self.load_order {
            let lib = self.libs.get(lib_path).unwrap();
            unsafe {
                if let Ok(value) = lib.get::<unsafe extern "C" fn()>(long_name.as_bytes()) {
                    let ptr = value.into_raw().into_raw() as *const c_void;
                    self.loaded_fns.insert(long_name.clone(), ptr);
                    debug!(
                        "Found native function {} in {}",
                        &long_name,
                        lib_path.display()
                    );
                    return Some(ptr);
                }

                if let Ok(value) = lib.get::<unsafe extern "C" fn()>(short_name.as_bytes()) {
                    let ptr = value.into_raw().into_raw() as *const c_void;
                    self.loaded_fns.insert(short_name.clone(), ptr);
                    debug!(
                        "Found native function {} in {}",
                        &short_name,
                        lib_path.display()
                    );
                    return Some(ptr);
                }
            }
        }

        None
    }
}
