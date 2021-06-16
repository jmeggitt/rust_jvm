use crate::jvm::bindings::{JNIEnv, jobject, jint, jstring};
use crate::jvm::{JVM, LocalVariable, clean_str};
use std::ffi::c_void;
use walkdir::WalkDir;

pub fn register_hooks(jvm: &mut JVM) {
    // Load classes since they are outside the class loaders visiblity
    // TODO: Maybe swap to a -cpstd/out/
    for entry in WalkDir::new("std/out") {
        let entry = entry.expect("Error reading stdlib");
        if entry.path().extension() == Some("class".as_ref()) {
            jvm.class_loader.load_new(&entry.path().to_path_buf()).unwrap();
        }
    }


    let print_fn = print_stream_hook as *const c_void;
    jvm.linked_libraries.register_fn("jvm/hooks/PrintStreamHook", "sendIO", "(ILjava/lang/String;)V", print_fn);

    jvm.locals[0] = LocalVariable::Int(0);
    let stdout = jvm.exec_static("jvm/hooks/PrintStreamHook", "buildStream", "(I)Ljava/io/PrintStream;").unwrap();
    jvm.locals[0] = LocalVariable::Int(1);
    let stderr = jvm.exec_static("jvm/hooks/PrintStreamHook", "buildStream", "(I)Ljava/io/PrintStream;").unwrap();

    jvm.init_class("java/lang/System");

    let field_reference = format!("{}_{}", clean_str("java/lang/System"), clean_str("out"));
    jvm.static_fields.insert(field_reference, stdout);

    let field_reference = format!("{}_{}", clean_str("java/lang/System"), clean_str("err"));
    jvm.static_fields.insert(field_reference, stderr);
}


pub unsafe extern "C" fn print_stream_hook(env: *mut JNIEnv, obj: jobject, fd: jint, str: jstring) {
    println!("Got print to {}", fd);
}






