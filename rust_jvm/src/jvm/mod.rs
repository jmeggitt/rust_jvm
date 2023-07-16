use std::io;
use std::option::Option::Some;
use std::sync::{Arc, Weak};

use jni::sys::{
    jchar, jint, JNI_VERSION_1_1, JNI_VERSION_1_2, JNI_VERSION_1_4, JNI_VERSION_1_6,
    JNI_VERSION_1_8,
};
use std::collections::{HashMap, HashSet};
use walkdir::WalkDir;

use crate::class::{Class, ClassLoader, MethodInfo};
use crate::jvm::call::{build_interface, CleanStr, NativeManager, VirtualMachine};
use crate::jvm::hooks::register_hooks;
use crate::jvm::mem::{
    ClassSchema, JavaValue, ManualInstanceReference, ObjectHandle, OBJECT_SCHEMA,
};
use crate::jvm::thread::{first_time_sys_thread_init, JavaThreadManager};
use parking_lot::RwLock;
use std::ffi::c_void;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::mem::size_of;
use std::ops::{Deref, Index, IndexMut};
use std::ptr::null;

pub mod call;
pub mod mem;

mod hooks;
mod internals;
pub mod thread;

// TODO: Review section 5.5 of the docs
pub struct JavaEnv {
    pub class_loader: ClassLoader,
    pub static_fields: StaticFields,
    pub static_load: HashSet<String>,
    pub linked_libraries: NativeManager,

    pub vm: VirtualMachine,

    pub registered_classes: HashMap<String, ObjectHandle>,

    pub thread_manager: JavaThreadManager,

    pub interned_strings: HashMap<String, ObjectHandle>,
    schemas: HashMap<String, Arc<ClassSchema>>,

    pub jni_vm: InvokeInterface,
}

#[repr(C)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct InvokeInterface {
    pub reserved0: Box<Weak<RwLock<JavaEnv>>>,
    pub reserved1: &'static (),
    pub reserved2: &'static (),
    pub DestroyJavaVM: extern "system" fn(vm: &RealJavaVM) -> jint,
    pub AttachCurrentThread:
        extern "system" fn(vm: &RealJavaVM, penv: *mut *mut c_void, args: *mut c_void) -> jint,
    pub DetachCurrentThread: extern "system" fn(vm: &RealJavaVM) -> jint,
    pub GetEnv: extern "system" fn(vm: &RealJavaVM, penv: *mut *mut c_void, version: jint) -> jint,
    pub AttachCurrentThreadAsDaemon:
        extern "system" fn(vm: &RealJavaVM, penv: *mut *mut c_void, args: *mut c_void) -> jint,
}

#[repr(C)]
pub struct RealJavaVM {
    inner: Box<InvokeInterface>,
}

impl Deref for RealJavaVM {
    type Target = InvokeInterface;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl RealJavaVM {
    extern "system" fn destroy_java_vm(&self) -> jint {
        unimplemented!()
    }

    extern "system" fn attach_current_thread(
        &self,
        _penv: *mut *mut c_void,
        _args: *mut c_void,
    ) -> jint {
        unimplemented!()
    }

    extern "system" fn detach_current_thread(&self) -> jint {
        unimplemented!()
    }

    extern "system" fn attach_current_thread_as_daemon(
        &self,
        _penv: *mut *mut c_void,
        _args: *mut c_void,
    ) -> jint {
        unimplemented!()
    }

    extern "system" fn get_env(&self, penv: *mut *mut c_void, version: jint) -> jint {
        match version {
            JNI_VERSION_1_1 => info!("Getting JNIEnv from JavaVM on JNI_VERSION_1_1"),
            JNI_VERSION_1_2 => info!("Getting JNIEnv from JavaVM on JNI_VERSION_1_2"),
            JNI_VERSION_1_4 => info!("Getting JNIEnv from JavaVM on JNI_VERSION_1_4"),
            JNI_VERSION_1_6 => info!("Getting JNIEnv from JavaVM on JNI_VERSION_1_6"),
            JNI_VERSION_1_8 => info!("Getting JNIEnv from JavaVM on JNI_VERSION_1_8"),
            x => error!("Unknown JNIEnv interface version: {:X}", x),
        };

        // let mut handle = (&*(self.reserved0 as *mut Arc<RwLock<JavaEnv>>)).clone();
        let handle = self.reserved0.clone();
        let interface = build_interface(&mut handle.upgrade().unwrap());
        unsafe {
            *penv = Box::leak(Box::new(Box::new(interface))) as *mut _ as *mut c_void;
        }
        version
    }
}

impl JavaEnv {
    pub fn new(class_loader: ClassLoader) -> Arc<RwLock<Self>> {
        #[cfg(feature = "thread_profiler")]
        thread_profiler::register_thread_with_profiler();
        assert_eq!(size_of::<Arc<RwLock<JavaEnv>>>(), size_of::<*mut c_void>());

        let mut jvm = Arc::new_cyclic(|this| {
            RwLock::new(JavaEnv {
                class_loader,
                static_fields: StaticFields::new(),
                static_load: HashSet::new(),
                linked_libraries: NativeManager::new(),
                vm: VirtualMachine::default(),
                registered_classes: HashMap::new(),
                thread_manager: JavaThreadManager::default(),
                interned_strings: HashMap::new(),
                schemas: HashMap::new(),
                jni_vm: InvokeInterface {
                    reserved0: Box::new(this.clone()),
                    reserved1: &(),
                    reserved2: &(),
                    DestroyJavaVM: RealJavaVM::destroy_java_vm,
                    AttachCurrentThread: RealJavaVM::attach_current_thread,
                    DetachCurrentThread: RealJavaVM::detach_current_thread,
                    GetEnv: RealJavaVM::get_env,
                    AttachCurrentThreadAsDaemon: RealJavaVM::attach_current_thread_as_daemon,
                },
            })
        });

        first_time_sys_thread_init(&mut jvm);

        Self::load_lib_by_name(&mut jvm, "java").unwrap();
        Self::load_lib_by_name(&mut jvm, "zip").unwrap();
        Self::load_lib_by_name(&mut jvm, "instrument").unwrap();
        register_hooks(&mut jvm);

        // warn!("Loading core")
        // Self::load_core_libs(&mut jvm).unwrap();

        jvm
    }

    pub fn dump_debug_info(&self) {
        let mut static_init = BufWriter::new(File::create("static_init.dump").unwrap());

        for field in &self.static_load {
            writeln!(&mut static_init, "{}", field).unwrap();
        }

        // let mut static_fields = BufWriter::new(File::create("static_fields.dump").unwrap());

        // for (k, v) in &self.static_fields {
        //     writeln!(&mut static_fields, "{}: {:?}", k, v).unwrap();
        // }
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
        // warn!("Class ptr:       {:p}", class.ptr());
        // warn!("Class as_ptr:    {:p}", class.as_ptr());
        // warn!("Instance as_ptr: {:p}", instance.as_ptr());
        let mut lock = instance.lock();

        self.class_loader.attempt_load(name).unwrap();

        lock.write_named_field("name", self.build_string(&name.replace('/', ".")));

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
    ) -> Option<(String, MethodInfo)> {
        // For arrays, defer to java/lang/Object
        if class.starts_with('[') {
            return self.find_instance_method("java/lang/Object", method, desc);
        }

        let entry_class = self.class_loader.class(class)?;

        if let Some(main_method) = entry_class.get_method(method, desc) {
            return Some((
                class.to_string(),
                main_method.clone(),
                // entry_class.constants.to_owned(),
            ));
        }

        if class != "java/lang/Object" {
            return self.find_instance_method(&entry_class.super_class(), method, desc);
        }

        None
    }

    pub fn load_lib_by_name(jvm: &mut Arc<RwLock<JavaEnv>>, name: &str) -> io::Result<()> {
        let lock = jvm.read();

        #[cfg(unix)]
        let lib_dir = lock.class_loader.class_path().java_home().join("lib/amd64");
        #[cfg(windows)]
        let lib_dir = lock.class_loader.class_path().java_home().join("bin");
        info!("Loading shared libraries from {}", lib_dir.display());

        #[cfg(windows)]
        unsafe {
            use std::ffi::CString;
            let path = CString::new(format!("{}", lib_dir.display())).unwrap();
            if winapi::um::winbase::SetDllDirectoryA(path.as_ptr()) == 0 {
                let err = winapi::um::errhandlingapi::GetLastError();
                panic!("Failed to set dll directory (error: {})", err);
            }
            debug!("SetDllDirectoryA({:?})", path);
        }

        let mut vm_ptr = &lock.jni_vm as *const _ as *const _;
        // Explicitly drop read lock to prevent lock from persisting until end of function and
        // blocking library load
        std::mem::drop(lock);

        let target_lib = if cfg!(windows) {
            lib_dir.join(format!("{}.dll", name))
        } else {
            lib_dir.join(format!("lib{}.so", name))
        };

        if !target_lib.exists() {
            error!("Dynamic Library not found: {}", target_lib.display());
            return Ok(());
        }

        match jvm.read().linked_libraries.load_library(target_lib) {
            Ok(Some(on_load_fn)) => unsafe {
                on_load_fn(&mut vm_ptr, null());
            },
            Ok(None) => warn!("Library is already loaded"),
            Err(err) => error!("{}", err),
        }
        // if let Err(e) = NativeManager::load_library(jvm.clone(), target_lib, &mut vm_ptr) {
        // if let Err(e) = NativeManager::load_library(jvm.clone(), target_lib, &mut vm_ptr) {
        //     error!("{}", e);
        // };

        Ok(())
    }

    // TODO: Split function into windows and unix versions?
    pub fn load_core_libs(jvm: &mut Arc<RwLock<JavaEnv>>) -> io::Result<()> {
        let lock = jvm.read();

        #[cfg(unix)]
        let lib_dir = lock.class_loader.class_path().java_home().join("lib");
        #[cfg(windows)]
        let lib_dir = lock.class_loader.class_path().java_home().join("bin");
        info!("Loading shared libraries from {}", lib_dir.display());

        #[cfg(windows)]
        unsafe {
            use std::ffi::CString;
            let path = CString::new(format!("{}", lib_dir.display())).unwrap();
            if winapi::um::winbase::SetDllDirectoryA(path.as_ptr()) == 0 {
                let err = winapi::um::errhandlingapi::GetLastError();
                panic!("Failed to set dll directory (error: {})", err);
            }
        }

        let mut vm_ptr = &lock.jni_vm as *const _ as *const _;
        // Explicitly drop read lock to prevent lock from persisting until end of function and
        // blocking library load
        std::mem::drop(lock);

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
                let jvm = jvm.read();
                let on_load = match jvm
                    .linked_libraries
                    .load_library(entry.path().to_path_buf())
                {
                    Ok(v) => v,
                    Err(e) => {
                        error!("{}", e);
                        None
                    }
                };
                std::mem::drop(jvm);
                if let Some(on_load_fn) = on_load {
                    unsafe {
                        on_load_fn(&mut vm_ptr, null());
                    }
                }
            }

            #[cfg(windows)]
            if entry.path().extension() == Some("dll".as_ref()) {
                let jvm = jvm.read();
                let on_load = match jvm
                    .linked_libraries
                    .load_library(entry.path().to_path_buf())
                {
                    Ok(v) => v,
                    Err(e) => {
                        error!("{}", e);
                        None
                    }
                };
                std::mem::drop(jvm);
                if let Some(on_load_fn) = on_load {
                    unsafe {
                        on_load_fn(&mut vm_ptr, null());
                    }
                }
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
        let mut lock = object.lock();

        let char_array = string
            .chars()
            .map(|x| x as u32 as jchar)
            .collect::<Vec<jchar>>();

        lock.write_named_field("value", Some(ObjectHandle::array_from_data(char_array)));
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

#[derive(Debug)]
pub struct StaticFields {
    static_obj: ObjectHandle,
    slots: HashMap<String, usize>,
    fields: Vec<JavaValue>,
}

impl StaticFields {
    pub fn new() -> Self {
        StaticFields {
            static_obj: ObjectHandle::new(OBJECT_SCHEMA.clone()),
            slots: Default::default(),
            fields: vec![],
        }
    }

    pub fn get_field_offset(&self, class: &str, field: &str) -> Option<usize> {
        let key = format!("{}_{}", CleanStr(&class), CleanStr(&field));
        self.slots.get(&key).copied()
    }

    pub fn set_static(&mut self, class: &str, field: &str, value: JavaValue) {
        let key = format!("{}_{}", CleanStr(&class), CleanStr(&field));
        if !self.slots.contains_key(&key) {
            self.slots.insert(key, self.fields.len());
            self.fields.push(value);
            return;
        }

        let idx = *self.slots.get(&key).unwrap();
        self.fields[idx] = value;
    }

    pub fn get_static(&self, class: &str, field: &str) -> Option<JavaValue> {
        self.get_field_offset(class, field).map(|x| self.fields[x])
    }
}

impl Index<usize> for StaticFields {
    type Output = JavaValue;

    fn index(&self, index: usize) -> &Self::Output {
        &self.fields[index]
    }
}

impl IndexMut<usize> for StaticFields {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.fields[index]
    }
}
