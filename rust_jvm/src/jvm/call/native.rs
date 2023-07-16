use crate::jvm::call::interface::build_interface;
use crate::jvm::call::FlowControl;
use crate::jvm::mem::{FieldDescriptor, JavaValue, ObjectHandle};
use crate::jvm::JavaEnv;
use jni::sys::{jint, JNINativeInterface_, JavaVM};
use libffi::middle::{Arg, Cif, CodePtr};
use libloading::Library;
use parking_lot::RwLock;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::Arc;

#[cfg(unix)]
use libloading::os::unix::Symbol;
#[cfg(windows)]
use libloading::os::windows::Symbol;

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

pub struct CleanStr<'a>(pub &'a str);

impl Display for CleanStr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for c in self.0.chars() {
            match c {
                '_' => write!(f, "_1")?,
                ';' => write!(f, "_2")?,
                '[' => write!(f, "_3")?,
                '/' => write!(f, "_")?,
                'a'..='z' | 'A'..='Z' | '0'..='9' => write!(f, "{}", c)?,
                x => write!(f, "_{:04x}", x as u32)?,
            }
        }
        Ok(())
    }
}

pub fn clean_desc(x: &str) -> CleanStr {
    assert!(x.starts_with('('));
    match x.find(')') {
        Some(end_pos) => CleanStr(&x[1..end_pos]),
        None => unreachable!(),
    }
}

pub type JniOnloadFn = unsafe extern "system" fn(*mut JavaVM, *const c_void) -> jint;
pub type JniFn = unsafe extern "system" fn();

struct LoadedLibrary {
    library: Library,
    path: PathBuf,
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct JniSymbolName<'a> {
    class: Cow<'a, str>,
    name: Cow<'a, str>,
    desc: Cow<'a, str>,
}

impl<'a> JniSymbolName<'a> {
    fn ensure_owned(self) -> JniSymbolName<'static> {
        JniSymbolName {
            class: Cow::Owned(self.class.into_owned()),
            name: Cow::Owned(self.name.into_owned()),
            desc: Cow::Owned(self.desc.into_owned()),
        }
    }
}

impl Display for JniSymbolName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Java_{}_{}__{}",
            CleanStr(&self.class),
            CleanStr(&self.name),
            clean_desc(&self.desc)
        )
    }
}

#[derive(Default)]
pub struct NativeManager {
    libraries: RwLock<Vec<LoadedLibrary>>,
    loaded_fns: RwLock<HashMap<JniSymbolName<'static>, *const c_void>>,
}

impl NativeManager {
    pub fn new() -> Self {
        let manager = NativeManager::default();
        use std::env::{current_dir, vars};
        info!("cwd: {:?}", current_dir().unwrap());
        info!("Environment variables:");
        for (key, value) in vars() {
            info!("\t{}: {}", key, value);
        }
        manager
    }

    pub fn load_library(
        &self,
        path: PathBuf,
    ) -> Result<Option<Symbol<JniOnloadFn>>, libloading::Error> {
        info!("Loading dynamic library {}", path.display());

        let mut libraries = self.libraries.write();
        if libraries.iter().any(|library| library.path == path) {
            debug!("Library {} is already loaded", path.display());
            return Ok(None);
        }

        let library = unsafe { Library::new(&path)? };

        let on_load_fn = unsafe {
            library
                .get::<JniOnloadFn>(b"JNI_OnLoad")
                .map(|symbol| symbol.into_raw())
        };

        libraries.push(LoadedLibrary { library, path });
        Ok(on_load_fn.ok())
    }

    pub fn find_symbol(&self, symbol_name: &[u8]) -> Option<Symbol<JniFn>> {
        self.libraries
            .read()
            .iter()
            .filter_map(|LoadedLibrary { library, path }| unsafe {
                let symbol = library.get::<JniFn>(symbol_name).ok()?;
                debug!(
                    "Found native function {} in {}",
                    String::from_utf8_lossy(symbol_name),
                    path.display()
                );
                Some(symbol.into_raw())
            })
            .next()
    }

    pub fn get_fn_ptr(&mut self, class: &str, name: &str, desc: &str) -> Option<*const c_void> {
        let jni_symbol_name = JniSymbolName {
            class: Cow::Borrowed(class),
            name: Cow::Borrowed(name),
            desc: Cow::Borrowed(desc),
        };

        if let Some(symbol) = self.loaded_fns.read().get(&jni_symbol_name) {
            return Some(*symbol);
        }

        let mut raw_symbol_name = Vec::new();
        if write!(
            &mut raw_symbol_name,
            "Java_{}_{}",
            CleanStr(class),
            CleanStr(name)
        )
        .is_err()
        {
            unreachable!()
        }

        let found_symbol = match self.find_symbol(&raw_symbol_name) {
            Some(symbol) => symbol.into_raw() as *mut c_void,
            None => {
                if write!(&mut raw_symbol_name, "__{}", clean_desc(desc)).is_err() {
                    unreachable!()
                }

                self.find_symbol(&raw_symbol_name)?.into_raw() as *mut c_void
            }
        };

        self.loaded_fns
            .write()
            .insert(jni_symbol_name.ensure_owned(), found_symbol);
        Some(found_symbol)
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

        assert!(!class.contains('.'));
        let symbol_name = JniSymbolName {
            class: Cow::Borrowed(class),
            name: Cow::Borrowed(name),
            desc: Cow::Borrowed(desc),
        };

        let mut loaded_fns = self.loaded_fns.write();
        if loaded_fns.contains_key(&symbol_name) {
            error!(
                "Failed to register native function! Already registered: {}",
                &symbol_name
            );
            return false;
        }

        loaded_fns
            .insert(symbol_name.ensure_owned(), fn_ptr)
            .is_none()
    }
}
