//! TODO: Split into seperate crate for shared library object

use std::ffi::c_void;

use jni::sys::{jboolean, jclass, jint, jobject, jstring, JNIEnv};

use crate::class::constant::ClassElement;
use crate::jvm::call::{clean_str, JavaEnvInvoke, NativeManager, RawJNIEnv};
use crate::jvm::internals::{register_method_handles_natives, unsafe_register_natives};
use crate::jvm::mem::{JavaValue, ObjectHandle};
use crate::jvm::JavaEnv;
use home::home_dir;
use parking_lot::RwLock;
use std::env::{current_dir, var};
use std::path::{Path, PathBuf};
use std::sync::Arc;

macro_rules! load_included_class {
    ($jvm:ident, $path:literal) => {
        let _bytes = include_bytes!(concat!(env!("OUT_DIR"), "/java_std/", $path));
        $jvm.write().class_loader.read_buffer(_bytes).unwrap();
    };
}

pub fn set_property(jvm: &mut Arc<RwLock<JavaEnv>>, obj: ObjectHandle, key: &str, value: &str) {
    let (k, v) = {
        let mut lock = jvm.write();
        (lock.build_string(key), lock.build_string(value))
    };

    let element = ClassElement {
        class: "java/util/Properties".to_string(),
        element: "setProperty".to_string(),
        desc: "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;".to_string(),
    };

    jvm.invoke_virtual(element, obj, vec![k, v]).unwrap();
}

pub fn build_system_properties(jvm: &mut Arc<RwLock<JavaEnv>>, obj: ObjectHandle) {
    set_property(jvm, obj, "java.version", "16.0.0");
    set_property(jvm, obj, "java.vendor", "jmeggitt");
    set_property(
        jvm,
        obj,
        "java.vendor.url",
        "https://github.com/jmeggitt/rust_jvm",
    );

    set_property(
        jvm,
        obj,
        "java.home",
        &format!(
            "{}",
            jvm.read().class_loader.class_path().java_home().display()
        ),
    );
    set_property(jvm, obj, "java.class.version", "60.0");

    // let class_path = jvm.read().class_loader.class_path().
    set_property(jvm, obj, "java.class.path", ".");

    set_property(
        jvm,
        obj,
        "java.specification.name",
        "Java Platform API Specification",
    );
    set_property(jvm, obj, "java.specification.vendor", "jmeggitt");
    set_property(jvm, obj, "java.specification.version", "16");

    set_property(jvm, obj, "os.name", std::env::consts::OS);
    set_property(jvm, obj, "os.arch", std::env::consts::ARCH);
    set_property(jvm, obj, "os.version", &whoami::distro()); // idk

    if cfg!(windows) {
        set_property(jvm, obj, "file.separator", "\\");
        set_property(jvm, obj, "path.separator", ";");
        set_property(jvm, obj, "line.separator", "\r\n");
    } else {
        set_property(jvm, obj, "file.separator", "/");
        set_property(jvm, obj, "path.separator", ":");
        set_property(jvm, obj, "line.separator", "\n");
    }

    set_property(jvm, obj, "user.name", &whoami::username());
    // error!("Language: {:?}", whoami::lang().collect::<Vec<String>>());
    // if let Some(language) = whoami::lang().next() {
    // }
    set_property(jvm, obj, "user.language", "en");

    set_property(
        jvm,
        obj,
        "user.home",
        &format!("{}", home_dir().unwrap_or_default().display()),
    );
    set_property(
        jvm,
        obj,
        "user.dir",
        &format!("{}", current_dir().unwrap_or_default().display()),
    );
    set_property(jvm, obj, "file.encoding", "utf-8");

    let library_path = if cfg!(unix) {
        let ld_path = var("LD_LIBRARY_PATH").unwrap_or_default();
        // default search directories on x86_64 linux. Most directories are expected to be symbolic links
        let default_path = "/usr/java/packages/lib:/usr/lib/x86_64-linux-gnu/jni:/lib/x86_64-linux-gnu:/usr/lib/x86_64-linux-gnu:/usr/lib/jni:/lib:/usr/lib";
        if ld_path.is_empty() {
            default_path.to_string()
        } else {
            format!("{}:{}", ld_path, default_path)
        }
    } else if cfg!(windows) {
        let mut library_search_path = String::new();

        if let Some(path) = std::env::current_exe()
            .ok()
            .map(|x| x.parent().unwrap().to_path_buf())
        {
            library_search_path.push_str(&format!("{}", path.display()));
            library_search_path.push(';');
        }

        // Default search locations
        library_search_path
            .push_str("C:\\Windows\\Sun\\Java\\bin;C:\\Windows\\system32;C:\\Windows");

        if let Ok(path) = var("PATH") {
            library_search_path.push(';');
            library_search_path.push_str(&path);
        }

        library_search_path.push_str(";.");
        library_search_path
    } else {
        error!("Unable to properly init library search path; unknown platform");
        String::new()
    };

    set_property(jvm, obj, "java.library.path", &library_path);

    if cfg!(target_endian = "little") {
        set_property(jvm, obj, "sun.cpu.endian", "little");
        set_property(jvm, obj, "sun.io.unicode.encoding", "UnicodeLittle");
    } else if cfg!(target_endian = "big") {
        set_property(jvm, obj, "sun.cpu.endian", "big");
        set_property(jvm, obj, "sun.io.unicode.encoding", "UnicodeBig");
    }

    if cfg!(target_pointer_width = "64") {
        set_property(jvm, obj, "sun.arch.data.model", "64");
    } else if cfg!(target_pointer_width = "32") {
        set_property(jvm, obj, "sun.arch.data.model", "32");
    } else if cfg!(target_pointer_width = "16") {
        set_property(jvm, obj, "sun.arch.data.model", "16");
    }

    let java_home = jvm.read().class_loader.class_path().java_home().to_owned();
    let library_path = if cfg!(windows) {
        java_home.join("bin")
    } else {
        java_home.join("lib")
    };
    set_property(
        jvm,
        obj,
        "sun.boot.library.path",
        &format!("{}", library_path.display()),
    );

    // let field_reference = format!("{}_{}", clean_str("java/lang/System"), clean_str("props"));
}

pub fn register_hooks(jvm: &mut Arc<RwLock<JavaEnv>>) {
    load_included_class!(jvm, "java/hooks/PrintStreamHook.class");

    // { CC"Java_sun_misc_Unsafe_registerNatives",                      NULL, FN_PTR(JVM_RegisterUnsafeMethods)       },
    // { CC"Java_java_lang_invoke_MethodHandleNatives_registerNatives", NULL, FN_PTR(JVM_RegisterMethodHandleMethods) },
    // { CC"Java_sun_misc_Perf_registerNatives",                        NULL, FN_PTR(JVM_RegisterPerfMethods)         },
    // { CC"Java_sun_hotspot_WhiteBox_registerNatives",                 NULL, FN_PTR(JVM_RegisterWhiteBoxMethods)     },

    jvm.write().linked_libraries.register_fn(
        "sun/misc/Unsafe",
        "registerNatives",
        "()V",
        unsafe_register_natives as *const c_void,
    );

    jvm.write().linked_libraries.register_fn(
        "java/lang/invoke/MethodHandleNatives",
        "registerNatives",
        "()V",
        register_method_handles_natives as *const c_void,
    );
    jvm.write().linked_libraries.register_fn(
        "sun/misc/Perf",
        "registerNatives",
        "()V",
        empty as *const c_void,
    );
    jvm.write().linked_libraries.register_fn(
        "sun/hotspot/Whitebox",
        "registerNatives",
        "()V",
        empty as *const c_void,
    );

    jvm.write().linked_libraries.register_fn(
        "java/lang/ClassLoader$NativeLibrary",
        "load",
        "(Ljava/lang/String;Z)V",
        load_library as *const c_void,
    );

    jvm.write().linked_libraries.register_fn(
        "java/hooks/PrintStreamHook",
        "sendIO",
        "(ILjava/lang/String;)V",
        Java_java_hooks_PrintStreamHook_sendIO as *const c_void,
    );

    // jvm.invoke_static(ClassElement::new("java/lang/System", "initProperties", "()V"), vec![]).unwrap();
    jvm.init_class("java/util/Properties");
    jvm.init_class("sun/misc/VM");
    let schema = jvm.write().class_schema("java/util/Properties");
    let obj = ObjectHandle::new(schema);

    jvm.invoke_virtual(
        ClassElement::new("java/util/Properties", "<init>", "()V"),
        obj,
        vec![],
    )
    .unwrap();

    jvm.write().static_fields.set_static(
        "java/lang/System",
        "props",
        JavaValue::Reference(Some(obj)),
    );

    build_system_properties(jvm, obj);
    let vm_props = jvm
        .read()
        .static_fields
        .get_static("sun/misc/VM", "savedProps")
        .unwrap()
        .expect_object();
    build_system_properties(jvm, vm_props);

    // Don't init stdout/stderr if doing tests to save time
    if cfg!(test) {
        return;
    }

    jvm.write()
        .class_loader
        .attempt_load("java/hooks/PrintStreamHook")
        .unwrap();
    let method = ClassElement::new(
        "java/hooks/PrintStreamHook",
        "buildStream",
        "(I)Ljava/io/PrintStream;",
    );

    let stdout = jvm
        .invoke_static(method.clone(), vec![JavaValue::Int(0)])
        .unwrap()
        .unwrap();

    let stderr = jvm
        .invoke_static(method, vec![JavaValue::Int(1)])
        .unwrap()
        .unwrap();

    let mut jvm = jvm.write();
    // let field_reference = format!("{}_{}", clean_str("java/lang/System"), clean_str("out"));
    jvm.static_fields
        .set_static("java/lang/System", "out", stdout);

    // let field_reference = format!("{}_{}", clean_str("java/lang/System"), clean_str("err"));
    jvm.static_fields
        .set_static("java/lang/System", "err", stderr);
}

pub unsafe extern "system" fn empty(_env: *mut JNIEnv, _cls: jclass) {}

#[no_mangle]
pub unsafe extern "system" fn Java_java_hooks_PrintStreamHook_sendIO(
    env: RawJNIEnv,
    _obj: jobject,
    fd: jint,
    string: jstring,
) {
    let output = obj_expect!(env, string).expect_string();

    if fd == 0 {
        print!("{}", output);
    } else if fd == 1 {
        eprint!("{}", output);
    }
}

#[no_mangle]
pub unsafe extern "system" fn load_library(
    env: RawJNIEnv,
    _cls: jclass,
    path: jstring,
    is_builtin: jboolean,
) {
    let path = obj_expect!(env, path).expect_string();
    info!(
        "java/lang/ClassLoader$NativeLibrary::load({:?}, {})",
        &path,
        is_builtin != 0
    );

    let jvm = env.read();
    let mut vm_ptr = &jvm.jni_vm as *const _;
    std::mem::drop(jvm);
    if let Err(e) = NativeManager::load_library(Arc::clone(&*env), PathBuf::from(path), &mut vm_ptr)
    {
        error!("{}", e);
    };
}

// C:\Program Files (x86)\Java\jdk1.8.0_291\bin
// C:\Windows\Sun\Java\bin
// C:\Windows\system32
// C:\Windows
//
// # System Path
// C:\Program Files\Common Files\Oracle\Java\javapath
// C:\Program Files (x86)\Common Files\Oracle\Java\javapath
// C:\Windows\system32
// C:\Windows
// C:\Windows\System32\Wbem
// C:\Windows\System32\WindowsPowerShell\v1.0\
// C:\Windows\System32\OpenSSH\
// C:\Program Files\Git\cmd
// C:\Program Files\dotnet\
// C:\Program Files\nodejs\
//
// # User Path
// C:\Users\Jasper\.cargo\bin
// C:\Users\Jasper\AppData\Local\Microsoft\WindowsApps
// C:\Users\Jasper\.dotnet\tools
// C:\Users\Jasper\AppData\Roaming\npm
//
// # Class path
// .
