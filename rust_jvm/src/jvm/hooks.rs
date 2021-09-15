//! TODO: Split into seperate crate for shared library object

use std::ffi::c_void;

use jni::sys::{jclass, JNIEnv};

use crate::constant_pool::ClassElement;
use crate::jvm::call::{clean_str, JavaEnvInvoke};
use crate::jvm::mem::{JavaValue, ObjectHandle};
use crate::jvm::JavaEnv;
use hashbrown::HashMap;
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

pub fn build_system_properties(jvm: &mut Arc<RwLock<JavaEnv>>) {
    jvm.init_class("java/util/Properties");
    let schema = jvm.write().class_schema("java/util/Properties");
    let obj = ObjectHandle::new(schema);

    jvm.invoke_virtual(
        ClassElement::new("java/util/Properties", "<init>", "()V"),
        obj,
        vec![],
    )
    .unwrap();

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
    set_property(jvm, obj, "os.version", std::env::consts::OS); // idk

    if cfg!(windows) {
        set_property(jvm, obj, "file.separator", "\\");
        set_property(jvm, obj, "path.separator", ";");
        set_property(jvm, obj, "line.separator", "\r\n");
    } else {
        set_property(jvm, obj, "file.separator", "/");
        set_property(jvm, obj, "path.separator", ":");
        set_property(jvm, obj, "line.separator", "\n");
    }

    set_property(
        jvm,
        obj,
        "user.name",
        &users::get_current_username()
            .map(|x| x.to_string_lossy().into_owned())
            .unwrap_or_default(),
    );
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

    let field_reference = format!("{}_{}", clean_str("java/lang/System"), clean_str("props"));
    jvm.write()
        .static_fields
        .insert(field_reference, JavaValue::Reference(Some(obj)));
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
    build_system_properties(jvm);

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
    let field_reference = format!("{}_{}", clean_str("java/lang/System"), clean_str("out"));
    jvm.static_fields.insert(field_reference, stdout);

    let field_reference = format!("{}_{}", clean_str("java/lang/System"), clean_str("err"));
    jvm.static_fields.insert(field_reference, stderr);
}

pub unsafe extern "system" fn empty(_env: *mut JNIEnv, _cls: jclass) {}
