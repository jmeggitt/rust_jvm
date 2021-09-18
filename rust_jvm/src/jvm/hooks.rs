//! TODO: Split into seperate crate for shared library object

use std::ffi::c_void;

use jni::sys::{jclass, jint, jobject, jstring, JNIEnv};

use crate::class::constant::ClassElement;
use crate::jvm::call::{clean_str, JavaEnvInvoke, RawJNIEnv};
use crate::jvm::mem::{JavaValue, ObjectHandle};
use crate::jvm::JavaEnv;
use home::home_dir;
use parking_lot::RwLock;
use std::env::current_dir;
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
    set_property(jvm, obj, "java.class.version", "60");
    // TODO: Use actual class path
    set_property(jvm, obj, "java.class.path", ".");

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
    if let Some(language) = whoami::lang().next() {
        set_property(jvm, obj, "user.language", &language);
    }
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

    // let field_reference = format!("{}_{}", clean_str("java/lang/System"), clean_str("props"));
}

pub fn register_hooks(jvm: &mut Arc<RwLock<JavaEnv>>) {
    load_included_class!(jvm, "java/hooks/PrintStreamHook.class");

    // TODO: Implement a register natives function for sun/misc/Unsafe and more declarations there
    jvm.write().linked_libraries.register_fn(
        "sun/misc/Unsafe",
        "registerNatives",
        "()V",
        empty as *const c_void,
    );
    jvm.write().linked_libraries.register_fn(
        "java/hooks/PrintStreamHook",
        "sendIO",
        "(ILjava/lang/String;)V",
        Java_java_hooks_PrintStreamHook_sendIO as *const c_void,
    );

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
