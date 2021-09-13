use std::io;
use std::option::Option::Some;
use std::sync::Arc;

use hashbrown::{HashMap, HashSet};
use jni::sys::jchar;
use walkdir::WalkDir;

use crate::class::{Class, ClassLoader, MethodInfo};
use crate::constant_pool::Constant;
use crate::jvm::call::{NativeManager, VirtualMachine};
use crate::jvm::hooks::register_hooks;
use crate::jvm::mem::{ClassSchema, JavaValue, ManualInstanceReference, ObjectHandle};
use crate::jvm::thread::JavaThreadManager;
use parking_lot::RwLock;

// macro_rules! fatal_error {
//     ($($arg:tt),*) => {{
//         error!($($arg),*);
//         panic!($($arg),*);
//     }};
// }

pub mod call;
pub mod mem;

mod hooks;
mod internals;
pub mod thread;

// TODO: Review section 5.5 of the docs
pub struct JavaEnv {
    pub class_loader: ClassLoader,
    pub static_fields: HashMap<String, JavaValue>,

    // Classes we have loaded and called <clinit> if possible
    pub static_load: HashSet<String>,
    // pub native_stack: OperandStack,
    pub linked_libraries: NativeManager,

    pub vm: VirtualMachine,

    pub registered_classes: HashMap<String, ObjectHandle>,

    // pub call_stack: Vec<(ObjectHandle, String)>,
    pub thread_manager: JavaThreadManager,
    // pub threads: HashMap<ThreadId, ObjectHandle>,
    // pub sys_thread_group: Option<ObjectHandle>,
    schemas: HashMap<String, Arc<ClassSchema>>,
}

impl JavaEnv {
    pub fn new(class_loader: ClassLoader) -> Arc<RwLock<Self>> {
        let mut jvm = JavaEnv {
            class_loader,
            static_fields: HashMap::new(),
            static_load: HashSet::new(),
            // native_stack: OperandStack::new(16384),
            linked_libraries: NativeManager::new(),
            vm: VirtualMachine::default(),
            // locals: vec![JavaValue::Int(0); 255],
            registered_classes: HashMap::new(),
            // call_stack: Vec::new(),
            // threads: HashMap::new(),
            // sys_thread_group: None,
            thread_manager: JavaThreadManager::default(),
            schemas: HashMap::new(),
        };

        #[cfg(feature = "thread_profiler")]
        thread_profiler::register_thread_with_profiler();
        // unsafe {
        //     jvm.make_primary_jvm();
        // }

        // Tell jvm to skip initializing the security manager because it requires a ton of native methods.
        // jvm.static_load.insert("java/lang/SecurityManager".to_string());

        // warn!("Loading core")
        jvm.load_core_libs().unwrap();
        let mut jvm = Arc::new(RwLock::new(jvm));
        register_hooks(&mut jvm);

        jvm
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
            return class.clone();
        }

        let schema = self.class_schema("java/lang/Class");
        let class = ObjectHandle::new(schema);
        let instance = class.expect_instance();

        instance.write_named_field("name", self.build_string(name));

        self.registered_classes
            .insert(name.to_string(), class.clone());
        class
    }

    pub fn instanceof(&self, instance: &str, target: &str) -> Option<bool> {
        if instance == target {
            return Some(true);
        }

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
                entry_class.old_constants().to_vec(),
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

        #[cfg(unix)]
        self.linked_libraries
            .load_library(
                "/mnt/c/Users/Jasper/CLionProjects/JavaClassTests/target/debug/librustyjvm.so"
                    .into(),
            )
            .unwrap();
        #[cfg(windows)]
        self.linked_libraries
            .load_library(
                "C:/Users/Jasper/CLionProjects/JavaClassTests/target/debug/rustyjvm.dll".into(),
            )
            .unwrap();

        // We need to load this first since the following libraries depend on it
        // #[cfg(unix)]
        // self.linked_libraries
        //     .load_library(lib_dir.join("amd64/server/libjvm.so"))?;
        // #[cfg(windows)]
        // self.linked_libraries
        //     .load_library(lib_dir.join("server/jvm.dll"))?;

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
                self.linked_libraries
                    .load_library(entry.path().to_path_buf())?;
            }
        }

        Ok(())
    }

    pub fn build_string(&mut self, string: &str) -> JavaValue {
        let mut handle = ObjectHandle::new(self.class_schema("java/lang/String").clone());
        let object = handle.expect_instance();

        let char_array = string
            .chars()
            .map(|x| x as u32 as jchar)
            .collect::<Vec<jchar>>();

        object.write_named_field("value", Some(ObjectHandle::array_from_data(char_array)));
        JavaValue::Reference(Some(handle))
    }

    fn expect_class(&mut self, class: &str) -> &Class {
        self.class_loader.attempt_load(class).unwrap();
        self.class_loader.class(class).unwrap()
    }

    pub fn debug_print_call_stack(&self) {
        debug!("Call stack:");
        self.thread_manager.debug_print_call_stack();
        // let mut padding = String::new();
        // for debug_str in &self.call_stack {
        //     debug!("{}{}", &padding, debug_str.1);
        //     padding.push_str("   ");
        // }
    }

    // pub fn entry_point(&mut self, class: &str, args: Vec<String>) -> io::Result<()> {
    //     info!(
    //         "Starting entry point class {} with arguments {:?}",
    //         class, &args
    //     );
    //
    //     self.class_loader.attempt_load(class)?;
    //
    //     let arg = JavaValue::Reference(Some(ObjectHandle::new_array::<Option<ObjectHandle>>(0)));
    //     // self.exec_static(class, "main", "([Ljava/lang/String;)V", vec![arg])
    //     //     .unwrap();
    //     let method = ClassElement::new(class, "main", "([Ljava/lang/String;)V");
    //     self.invoke_static(method, vec![arg]).unwrap();
    //
    //     Ok(())
    // }
}
