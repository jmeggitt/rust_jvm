use std::io;
use std::option::Option::Some;
use std::sync::Arc;

use hashbrown::{HashMap, HashSet};
use jni::sys::jchar;
use walkdir::WalkDir;

// use crate::class::{Class, ClassLoader, MethodInfo};
// use crate::constant_pool::Constant;
use crate::class::constant::Constant;
use crate::class::{Class, ClassLoader, MethodInfo};
use crate::jvm::call::{NativeManager, VirtualMachine};
use crate::jvm::hooks::register_hooks;
use crate::jvm::mem::{ClassSchema, JavaValue, ManualInstanceReference, ObjectHandle};
use crate::jvm::thread::{first_time_sys_thread_init, JavaThreadManager};
use parking_lot::RwLock;
use std::ffi::CString;
use std::fs::File;
use std::io::{BufWriter, Write};

pub mod call;
pub mod mem;

mod hooks;
mod internals;
pub mod thread;

// TODO: Review section 5.5 of the docs
pub struct JavaEnv {
    pub class_loader: ClassLoader,
    pub static_fields: HashMap<String, JavaValue>,
    pub static_load: HashSet<String>,
    pub linked_libraries: NativeManager,

    pub vm: VirtualMachine,

    pub registered_classes: HashMap<String, ObjectHandle>,

    pub thread_manager: JavaThreadManager,

    pub interned_strings: HashMap<String, ObjectHandle>,
    schemas: HashMap<String, Arc<ClassSchema>>,
}

impl JavaEnv {
    pub fn new(class_loader: ClassLoader) -> Arc<RwLock<Self>> {
        let mut jvm = JavaEnv {
            class_loader,
            static_fields: HashMap::new(),
            static_load: HashSet::new(),
            linked_libraries: NativeManager::new(),
            vm: VirtualMachine::default(),
            registered_classes: HashMap::new(),
            thread_manager: JavaThreadManager::default(),
            interned_strings: HashMap::new(),
            schemas: HashMap::new(),
        };

        #[cfg(feature = "thread_profiler")]
        thread_profiler::register_thread_with_profiler();

        // warn!("Loading core")
        jvm.load_core_libs().unwrap();
        let mut jvm = Arc::new(RwLock::new(jvm));

        first_time_sys_thread_init(&mut jvm);
        register_hooks(&mut jvm);

        jvm
    }

    pub fn dump_debug_info(&self) {
        let mut static_init = BufWriter::new(File::create("static_init.dump").unwrap());

        for field in &self.static_load {
            writeln!(&mut static_init, "{}", field).unwrap();
        }

        let mut static_fields = BufWriter::new(File::create("static_fields.dump").unwrap());

        for (k, v) in &self.static_fields {
            writeln!(&mut static_fields, "{}: {:?}", k, v).unwrap();
        }
    }

    pub fn class_schema(&mut self, class: &str) -> Arc<ClassSchema> {
        if !self.schemas.contains_key(class) {
            let class_spec = self.expect_class(class).clone();

            let schema = ClassSchema::build(&class_spec, self);
            self.schemas.insert(class.to_string(), Arc::new(schema));
        }

        self.schemas.get(class).cloned().unwrap()
    }

    pub fn class_instance(&mut self, name: &str) -> ObjectHandle {
        if let Some(class) = self.registered_classes.get(name) {
            return *class;
        }

        let schema = self.class_schema("java/lang/Class");
        let class = ObjectHandle::new(schema);
        let instance = class.expect_instance();

        self.class_loader.attempt_load(name).unwrap();

        instance.write_named_field("name", self.build_string(&name.replace('/', ".")));

        self.registered_classes.insert(name.to_string(), class);
        class
    }

    pub fn instanceof(&self, instance: &str, target: &str) -> Option<bool> {
        if instance == target || target == "java/lang/Object" {
            return Some(true);
        }

        // If this is a regular object, we hit the base case
        if instance == "java/lang/Object" {
            return Some(false);
        }

        // TODO: Arrays currently do not hold their types correctly for objects
        if instance.starts_with("[L") {
            return Some(target.starts_with("[L"));
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
        // For arrays, defer to java/lang/Object
        if class.starts_with('[') {
            return self.find_instance_method("java/lang/Object", method, desc);
        }

        let entry_class = self.class_loader.class(class)?;

        if let Some(main_method) = entry_class.get_method(method, desc) {
            return Some((
                class.to_string(),
                main_method.clone(),
                entry_class.constants.to_owned(),
            ));
        }

        if class != "java/lang/Object" {
            return self.find_instance_method(&entry_class.super_class(), method, desc);
        }

        None
    }

    // TODO: Split function into windows and unix versions?
    pub fn load_core_libs(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        let lib_dir = self.class_loader.class_path().java_home().join("lib");
        #[cfg(windows)]
        let lib_dir = self.class_loader.class_path().java_home().join("bin");
        info!("Loading shared libraries from {}", lib_dir.display());

        #[cfg(windows)]
        unsafe {
            let path = CString::new(format!("{}", lib_dir.display())).unwrap();
            if winapi::um::winbase::SetDllDirectoryA(path.as_ptr()) == 0 {
                let err = winapi::um::errhandlingapi::GetLastError();
                panic!("Failed to set dll directory (error: {})", err);
            }
        }

        // Load includes in deterministic order to ensure regularity between runs
        for entry in WalkDir::new(&lib_dir).sort_by_file_name() {
            let entry = entry?;
            if !entry.path().is_file() {
                continue;
            }

            #[cfg(unix)]
            if entry.path().to_str().unwrap().contains("libjvm") {
                continue;
            }

            #[cfg(unix)]
            if entry.path().extension() == Some("so".as_ref()) {
                self.linked_libraries
                    .load_library(entry.path().to_path_buf())?;
            }

            #[cfg(windows)]
            if entry.path().extension() == Some("dll".as_ref()) {
                if let Err(e) = self
                    .linked_libraries
                    .load_library(entry.path().to_path_buf())
                {
                    error!("{}", e);
                };
            }
        }

        Ok(())
    }

    pub fn build_string(&mut self, string: &str) -> JavaValue {
        if self.interned_strings.contains_key(string) {
            return JavaValue::Reference(Some(*self.interned_strings.get(string).unwrap()));
        }

        let handle = ObjectHandle::new(self.class_schema("java/lang/String"));
        let object = handle.expect_instance();

        let char_array = string
            .chars()
            .map(|x| x as u32 as jchar)
            .collect::<Vec<jchar>>();

        object.write_named_field("value", Some(ObjectHandle::array_from_data(char_array)));
        self.interned_strings.insert(string.to_string(), handle);
        JavaValue::Reference(Some(handle))
    }

    fn expect_class(&mut self, class: &str) -> &Class {
        self.class_loader.attempt_load(class).unwrap();
        self.class_loader.class(class).unwrap()
    }

    pub fn debug_print_call_stack(&self) {
        debug!("Call stack:");
        self.thread_manager.debug_print_call_stack();
    }
}
