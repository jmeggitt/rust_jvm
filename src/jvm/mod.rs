use crate::class::{Class, ClassLoader, AccessFlags, BufferedRead, MethodInfo};
use hashbrown::{HashMap, HashSet};
use std::io;
use std::rc::Rc;
use crate::jvm::stack::OperandStack;
use libloading::Library;
use std::ffi::c_void;
use std::path::PathBuf;
use std::io::{Error, ErrorKind, Cursor};
use std::fs::read_dir;
use walkdir::WalkDir;
use std::env::{set_current_dir, current_dir, set_var, var};
use crate::types::FieldDescriptor;
use crate::jvm::bindings::{jvalue, _jobject};
use std::ptr::{null, null_mut};
use std::os::raw::c_long;
use std::option::Option::Some;

macro_rules! fatal_error {
    ($($arg:tt),*) => {{
        error!($($arg),*);
        panic!($($arg),*);
    }};
}

/// This was generated from jni.h so allow any variations
#[allow(warnings)]
pub mod bindings;

mod mem;

pub use mem::*;
use crate::jvm::hooks::register_hooks;
use crate::attribute::CodeAttribute;
use crate::constant_pool::Constant;

mod stack;
mod interface;
mod hooks;

#[cfg(unix)]
mod exec;


pub struct StackFrame {
    // Either an object or class
    target: Object,
    // Comparable to the .text section of a binary
    constants: Vec<Constant>,
    // Values treated as registers
    locals: Vec<LocalVariable>,
    // The stack frame
    stack: Vec<LocalVariable>,
    // Instruction pointer
    rip: usize,
    // Instructions within function
    code: CodeAttribute,
    // Work around so instructions can set the return value
    returns: Option<LocalVariable>,
}

impl StackFrame {

    pub fn new(target: Object, method: MethodInfo, constants: Vec<Constant>, args: Vec<LocalVariable>) -> Self {
        if main_method.access.contains(AccessFlags::NATIVE) {
            panic!("Attempted to create stack frame for native method!");
        }

        let code = method.code(&constants);

        let mut locals = vec![LocalVariable::Int(0); code.max_locals as usize];
        for (idx, value) in args.into_iter().enumerate() {
            locals[idx] = value;
        }

        StackFrame {
            target,
            constants,
            locals,
            stack: Vec::with_capacity(code.max_stack as usize),
            rip: 0,
            code,
            returns: None
        }
    }

    pub fn exec(&mut self, jvm: &mut JVM) -> Option<LocalVariable> {
        loop {
            let start_rip = self.rip;
            let instruction = &self.code.instructions[self.rip];
            instruction.exec(&mut stack, &constants, jvm);

            match &self.returns {
                Some(LocalVariable::Padding) => return None,
                Some(v) => return Some(v.clone()),
                _ => {},
            };

            // The instruction was not a jump so we can increment this field normally.
            if start_rip == self.rip {
                self.rip += 1;
            }
        }

        None
    }
}


pub struct JVM {
    pub class_loader: ClassLoader,
    pub static_fields: HashMap<String, LocalVariable>,

    // Classes we have loaded and called <clinit> if possible
    pub static_load: HashSet<String>,
    pub native_stack: OperandStack,
    pub linked_libraries: NativeManager,

    // Basically just registers
    pub locals: Vec<LocalVariable>,
}


impl JVM {
    pub fn new(class_loader: ClassLoader) -> Self {
        JVM {
            class_loader,
            static_fields: HashMap::new(),
            static_load: HashSet::new(),
            native_stack: OperandStack::new(16384),
            linked_libraries: NativeManager::new(),
            locals: vec![LocalVariable::Int(0); 255],
        }
    }

    pub fn find_instance_method(&self, class: &str, method: &str, desc: &str) -> Option<MethodInfo> {
        let entry_class = self.class_loader.class(class)?;

        if let Some(main_method) = entry_class.get_method(method, desc) {
            return Some(main_method.clone())
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
            self.linked_libraries.load_library(lib_dir.join("amd64/server/libjvm.so"))?;
        #[cfg(windows)]
            self.linked_libraries.load_library(lib_dir.join("bin/server/jvm.dll"))?;

        // Load includes in deterministic order to ensure regularity between runs
        for entry in WalkDir::new(&lib_dir).sort_by_file_name() {
            let entry = entry?;
            if !entry.path().is_file() {
                continue;
            }

            #[cfg(unix)]
            if entry.path().extension() == Some("so".as_ref()) {
                self.linked_libraries.load_library(entry.path().to_path_buf())?;
            }

            #[cfg(windows)]
            if entry.path().extension() == Some("dll".as_ref()) {
                self.linked_libraries.load_library(entry.path().to_path_buf())?;
            }
        }

        Ok(())
    }

    pub unsafe fn make_primary_jvm(&mut self) {
        let ptr = self as *mut Self;
        interface::GLOBAL_JVM = Some(Box::from_raw(ptr));
    }

    fn expect_class(&mut self, class: &str) -> &Class {
        self.class_loader.class(class).unwrap()
    }

    pub fn exec_static(
        &mut self,
        class: &str,
        method: &str,
        desc: &str,
    ) -> Option<LocalVariable> {
        debug!("Executing static {}::{} {}", class, method, desc);

        let mut stack_frame: Vec<LocalVariable> = Vec::new();
        let entry_class = match self.class_loader.class(class) {
            Some(v) => v,
            None => fatal_error!("Unable to load class {}", class),
        };

        let main_method = match entry_class.get_method(method, desc) {
            Some(v) => v,
            None => fatal_error!("Class {} does not contain {} {}", class, method, desc),
        };

        let constants = entry_class.constants().to_vec();

        if main_method.access.contains(AccessFlags::NATIVE) {
            let descriptor = main_method.descriptor(&constants).unwrap();

            let fn_ptr = match self.linked_libraries.get_fn_ptr(class, method, desc) {
                Some(v) => v,
                None => panic!("Unable to find function ptr {}::{} {}", class, method, desc),
            };

            let mut buffer = Cursor::new(descriptor.as_bytes().to_vec());
            if let Ok(FieldDescriptor::Method { args, returns }) = FieldDescriptor::read(&mut buffer) {
                let mut stack = Vec::new();

                for _ in 0..args.len().min(stack_frame.len()) {
                    if let Some(v) = stack_frame.pop().unwrap().into() {
                        stack.push(v);
                    }
                }
                stack.reverse();

                let ret = unsafe {
                    let owned_class_name = class.to_string();
                    let class_object = &owned_class_name as *const String as *mut _jobject;

                    self.native_stack.native_method_call(fn_ptr, class_object, stack)
                };

                if let Some(v) = returns.cast(ret) {
                    stack_frame.push(v);
                }
            }
        } else {
            let code = main_method.code(entry_class.constants());

            trace!("Expecting {} local variables", code.max_locals);
            // assert!(code.max_locals as usize <= local_variables.len());
            let mut stack = Vec::new();

            for _ in 0..code.max_locals.min(stack_frame.len() as u16) {
                stack.push(stack_frame.pop().unwrap());
            }
            stack.reverse();

            for instruction in &code.instructions {
                instruction.exec(&mut stack, &constants, self);
            }
        }

        None
    }

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
                self.exec_static(class, "<clinit>", "()V");
            }
        }
    }

    pub fn entry_point(&mut self, class: &str, args: Vec<String>) -> io::Result<()> {
        info!(
            "Starting entry point class {} with arguments {:?}",
            class, &args
        );

        register_hooks(self);

        self.class_loader.attempt_load(class)?;

        // This really should be an instance of a 0 length array
        // LocalVariable::Reference(None));
        self.locals[0] = LocalVariable::Reference(Some(Rc::new(Object::Array {
            values: vec![],
            element_type: FieldDescriptor::Object("java/lang/String".to_string()),
        })));

        self.exec_static(class, "main", "([Ljava/lang/String;)V");

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


pub struct NativeManager {
    libs: HashMap<PathBuf, Library>,
    load_order: Vec<PathBuf>,
    loaded_fns: HashMap<String, *const c_void>,
}

impl NativeManager {
    pub fn new() -> Self {
        NativeManager {
            libs: HashMap::new(),
            load_order: Vec::new(),
            loaded_fns: HashMap::new(),
        }
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
        Some(clean_str(&x[1..x.find(")")?]))
    }

    pub fn register_fn(&mut self, class: &str, name: &str, desc: &str, fn_ptr: *const c_void) -> bool {
        debug!("Registering native function for {}::{} {}", class, name, desc);
        let long_name = format!("Java_{}_{}__{}", clean_str(class), clean_str(name), Self::clean_desc(desc).unwrap());
        self.loaded_fns.insert(long_name, fn_ptr).is_none()
    }

    pub fn get_fn_ptr(&mut self, class: &str, name: &str, desc: &str) -> Option<*const c_void> {
        let long_name = format!("Java_{}_{}__{}", clean_str(class), clean_str(name), Self::clean_desc(desc)?);
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
                    debug!("Found native function {} in {}", &long_name, lib_path.display());
                    return Some(ptr);
                }

                if let Ok(value) = lib.get::<unsafe extern "C" fn()>(short_name.as_bytes()) {
                    let ptr = value.into_raw().into_raw() as *const c_void;
                    self.loaded_fns.insert(short_name.clone(), ptr);
                    debug!("Found native function {} in {}", &short_name, lib_path.display());
                    return Some(ptr);
                }
            }
        }

        None
    }
}

