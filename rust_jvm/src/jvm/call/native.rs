use crate::jvm::call::interface::build_interface;
use crate::jvm::call::FlowControl;
use crate::jvm::mem::{FieldDescriptor, JavaValue, ObjectHandle};
use crate::jvm::{internals, JavaEnv};
use hashbrown::HashMap;
use jni::sys::{jint, JNINativeInterface_, JavaVM};
use libffi::middle::{Arg, Cif, CodePtr};
use libloading::Library;
use parking_lot::RwLock;
use std::ffi::c_void;
use std::io;
use std::io::{Error, ErrorKind};
use std::mem::transmute;
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::sync::Arc;

pub struct NativeCall {
    cif: Cif,
    fn_ptr: CodePtr,
    desc: FieldDescriptor,
}

// Keep a global null value so I can take a pointer to a null value
const NULL_BOX: *mut c_void = null_mut();

impl NativeCall {
    pub fn new(fn_ptr: *const c_void, desc: FieldDescriptor) -> Self {
        assert!(matches!(&desc, FieldDescriptor::Method { .. }));

        let cif = desc.build_cif();
        NativeCall {
            cif,
            fn_ptr: CodePtr::from_ptr(fn_ptr),
            desc,
        }
    }

    fn verify_args(&self, arguments: &[JavaValue]) -> Vec<JavaValue> {
        if let FieldDescriptor::Method { args, .. } = &self.desc {
            let mut ret = Vec::new();
            let mut idx = 0;
            for desc in args {
                match desc {
                    FieldDescriptor::Long => {
                        assert!(matches!(&arguments[idx], JavaValue::Long(_)));
                        ret.push(arguments[idx]);
                        idx += 2;
                    }
                    FieldDescriptor::Double => {
                        assert!(matches!(&arguments[idx], JavaValue::Double(_)));
                        ret.push(arguments[idx]);
                        idx += 2;
                    }
                    x => {
                        if let Some(x) = x.assign_from(arguments[idx]) {
                            ret.push(x);
                            idx += 1;
                        } else {
                            panic!("Arguments passed do not match those for native call")
                        }
                    }
                }
            }

            ret
            // for (desc, arg) in args.iter().zip(arguments) {
            //     assert!(desc.matches(arg));
            // }
        } else {
            panic!("Arguments do not match function!")
        }
    }

    fn wrap_arg(local: &JavaValue) -> Arg {
        match local {
            JavaValue::Byte(x) => Arg::new(x),
            JavaValue::Char(x) => Arg::new(x),
            JavaValue::Short(x) => Arg::new(x),
            JavaValue::Int(x) => Arg::new(x),
            JavaValue::Float(x) => Arg::new(x),
            JavaValue::Reference(None) => Arg::new(&NULL_BOX),
            JavaValue::Reference(Some(x)) => Arg::new(x),
            JavaValue::Long(x) => Arg::new(x),
            JavaValue::Double(x) => Arg::new(x),
        }
    }

    pub fn exec(
        &self,
        jvm: &mut Arc<RwLock<JavaEnv>>,
        target: ObjectHandle,
        mut args: Vec<JavaValue>,
    ) -> Result<Option<JavaValue>, FlowControl> {
        args = self.verify_args(&args);

        let jni_env = build_interface(jvm);
        let target_ptr = target.ptr();

        let jni_envp = &jni_env as *const JNINativeInterface_;
        let jni_envpp = &jni_envp as *const *const JNINativeInterface_;

        let mut ffi_args = Vec::with_capacity(args.len() + 2);
        ffi_args.push(Arg::new(&jni_envpp));
        ffi_args.push(Arg::new(&target_ptr));

        for arg in &args {
            ffi_args.push(NativeCall::wrap_arg(arg));
        }

        if let FieldDescriptor::Method { returns, .. } = &self.desc {
            let ret = unsafe {
                Ok(Some(match &**returns {
                    FieldDescriptor::Void => {
                        self.cif.call::<c_void>(self.fn_ptr, &ffi_args);
                        return Ok(None);
                    }
                    FieldDescriptor::Byte => JavaValue::Byte(self.cif.call(self.fn_ptr, &ffi_args)),
                    FieldDescriptor::Char => JavaValue::Char(self.cif.call(self.fn_ptr, &ffi_args)),
                    FieldDescriptor::Double => {
                        JavaValue::Double(self.cif.call(self.fn_ptr, &ffi_args))
                    }
                    FieldDescriptor::Float => {
                        JavaValue::Float(self.cif.call(self.fn_ptr, &ffi_args))
                    }
                    FieldDescriptor::Int => JavaValue::Int(self.cif.call(self.fn_ptr, &ffi_args)),
                    FieldDescriptor::Long => JavaValue::Long(self.cif.call(self.fn_ptr, &ffi_args)),
                    FieldDescriptor::Short => {
                        JavaValue::Short(self.cif.call(self.fn_ptr, &ffi_args))
                    }
                    FieldDescriptor::Boolean => {
                        JavaValue::Byte(self.cif.call(self.fn_ptr, &ffi_args))
                    }
                    FieldDescriptor::Object(_) | FieldDescriptor::Array(_) => {
                        let ret = self.cif.call(self.fn_ptr, &ffi_args);
                        JavaValue::Reference(ObjectHandle::from_ptr(ret))
                    }
                    _ => panic!(),
                }))
            };

            let mut lock = jvm.write();
            if let Some(exception) = lock.thread_manager.get_sticky_exception() {
                // Clear and propogate exception
                lock.thread_manager.set_sticky_exception(None);
                return Err(FlowControl::Throws(Some(exception)));
            }
            ret
        } else {
            unreachable!("Should have passed argument check")
        }
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

unsafe impl Send for NativeManager {}
unsafe impl Sync for NativeManager {}

impl NativeManager {
    pub fn new() -> Self {
        let mut manager = NativeManager::default();
        use std::env::{current_dir, vars};
        info!("cwd: {:?}", current_dir().unwrap());
        info!("Environment variables:");
        for (key, value) in vars() {
            info!("\t{}: {}", key, value);
        }
        // internals::register_natives(&mut manager);
        manager
    }

    pub fn load_library(
        jvm: Arc<RwLock<JavaEnv>>,
        path: PathBuf,
        vm: *mut JavaVM,
    ) -> io::Result<()> {
        info!("Loading dynamic library {}", path.display());
        let lock = jvm.read();
        let has_library = lock.linked_libraries.libs.contains_key(&path);
        std::mem::drop(lock);
        if !has_library {
            unsafe {
                match Library::new(&path) {
                    Ok(v) => {
                        let mut lock = jvm.write();
                        lock.linked_libraries.load_order.push(path.clone());
                        lock.linked_libraries.libs.insert(path.clone(), v)
                    }
                    Err(e) => {
                        return Err(Error::new(
                            ErrorKind::Other,
                            format!(
                                "{}: {} ({})",
                                &e,
                                std::error::Error::source(&e).unwrap(),
                                path.display()
                            ),
                        ));
                    }
                };

                let load_fn = jvm
                    .read()
                    .linked_libraries
                    .libs
                    .get(&path)
                    .unwrap()
                    .get::<unsafe extern "C" fn()>(b"JNI_OnLoad")
                    .map(|x| x.into_raw().into_raw() as *mut c_void);
                if let Ok(onload) = load_fn {
                    info!("Running JNI_OnLoad for {}", path.display());
                    let onload: unsafe extern "system" fn(*mut JavaVM, *const c_void) -> jint =
                        transmute(onload);
                    onload(vm, null());
                }
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
            clean_str(&class.replace('.', "/")),
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
        debug!("Searching for function {}", &long_name);

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
