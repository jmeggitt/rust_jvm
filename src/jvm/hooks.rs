//! TODO: Split into seperate crate for shared library object

use std::cell::{RefCell, UnsafeCell};
use std::collections::hash_map::DefaultHasher;
use std::ffi::c_void;
use std::hash::{Hash, Hasher};
use std::io;
use std::io::Cursor;
use std::mem::{forget, ManuallyDrop, transmute};
use std::rc::Rc;

use jni::sys::{jboolean, jclass, jint, JNIEnv, jobject, jstring};
use walkdir::WalkDir;

use crate::instruction::Instruction;
use crate::jvm::{clean_str, JVM, LocalVariable, Object};
use crate::jvm::interface::GLOBAL_JVM;

pub fn register_hooks(jvm: &mut JVM) {
    // Load classes since they are outside the class loaders visiblity
    // TODO: Maybe swap to a -cpstd/out/
    for entry in WalkDir::new("std/out") {
        let entry = entry.expect("Error reading stdlib");
        if entry.path().extension() == Some("class".as_ref()) {
            jvm.class_loader
                .load_new(&entry.path().to_path_buf())
                .unwrap();
        }
    }

    jvm.linked_libraries.register_fn(
        "sun/misc/Unsafe",
        "registerNatives",
        "()V",
        empty as *const c_void,
    );


    // jvm.init_class("java/lang/Object");


    jvm.linked_libraries.register_fn(
        "java/lang/Object",
        "hashCode",
        "()I",
        hash_object as *const c_void,
    );




    jvm.init_class("java/lang/System");

    jvm.linked_libraries.register_fn(
        "java/lang/System",
        "arraycopy",
        "(Ljava/lang/Object;ILjava/lang/Object;II)V",
        array_copy as *const c_void,
    );

    jvm.init_class("java/lang/Class");

    jvm.linked_libraries.register_fn(
        "java/lang/Class",
        "desiredAssertionStatus0",
        "(Ljava/lang/Class;)Z",
        desired_assertions as *const c_void,
    );

    jvm.linked_libraries.register_fn(
        "java/lang/Class",
        "getPrimitiveClass",
        "(Ljava/lang/String;)Ljava/lang/Class;",
        get_class as *const c_void,
    );

    let print_fn = print_stream_hook as *const c_void;
    jvm.linked_libraries.register_fn(
        "jvm/hooks/PrintStreamHook",
        "sendIO",
        "(ILjava/lang/String;)V",
        print_fn,
    );

    // jvm.locals[0] = LocalVariable::Int(0);
    let stdout = jvm
        .exec_static(
            "jvm/hooks/PrintStreamHook",
            "buildStream",
            "(I)Ljava/io/PrintStream;",
            vec![LocalVariable::Int(0)],
        )
        .unwrap().unwrap();
    // jvm.locals[0] = LocalVariable::Int(1);
    let stderr = jvm
        .exec_static(
            "jvm/hooks/PrintStreamHook",
            "buildStream",
            "(I)Ljava/io/PrintStream;",
            vec![LocalVariable::Int(1)],
        )
        .unwrap().unwrap();


    let field_reference = format!("{}_{}", clean_str("java/lang/System"), clean_str("out"));
    jvm.static_fields.insert(field_reference, stdout);

    let field_reference = format!("{}_{}", clean_str("java/lang/System"), clean_str("err"));
    jvm.static_fields.insert(field_reference, stderr);
}

pub unsafe extern "C" fn hash_object(env: *mut JNIEnv, obj: jobject) -> jint {
    let mut hasher = DefaultHasher::new();
    let name_object = &*(obj as *const Rc<UnsafeCell<Object>>);
    (&*name_object.get()).hash(&mut hasher);
    let result = hasher.finish();
    let [a, b] = transmute::<_, [i32; 2]>(result);
    a ^ b
}

pub unsafe extern "C" fn array_copy(env: *mut JNIEnv, cls: jclass, src: jobject, src_pos: jint, dst: jobject, dst_pos: jint, length: jint) {
    debug!("Got correct version of arraycopy with src: {:p} dst: {:p}", src, dst);
    let cls_object = &*(cls as *const Rc<UnsafeCell<Object>>);
    debug!("Got class object: {:?}", cls_object);

    let src_object = &*(&*(src as *const Rc<UnsafeCell<Object>>)).get();
    let mut dst_object = &mut *(&mut *(dst as *mut Rc<UnsafeCell<Object>>)).get();

    let src_vec = match &*src_object {
        Object::Array { values, .. } => values,
        x => panic!("Attempted to call arraycopy with non array entries: {:?}", x),
    };

    let dst_vec  = match &mut *dst_object {
        Object::Array { values, .. } => values,
        x => panic!("Attempted to call arraycopy with non array entries: {:?}", x),
    };

    // Be lazy since we need to clone each element
    for offset in 0..length as usize {
        dst_vec[dst_pos as usize + offset] = src_vec[src_pos as usize + offset].clone();
    }
}


pub unsafe extern "C" fn empty(env: *mut JNIEnv, cls: jclass) {}


pub unsafe extern "C" fn desired_assertions(env: *mut JNIEnv, cls: jclass, target: jclass) -> jboolean {
    0 // Don't do assertions, I don't need the extra work
}

pub unsafe extern "C" fn get_class(env: *mut JNIEnv, cls: jclass, name: jstring) -> jclass {
    debug!("Executing getPrimitiveClass");
    let name_object = &*(name as *const Rc<UnsafeCell<Object>>);
    debug!("Received object: {:p}", name);
    debug!("Object: {:p}, debug: {:?}", name, name_object);

    let name = (&*name_object.get()).expect_string();
    info!("Getting class named {:?}", name);

    let class = GLOBAL_JVM.as_mut().unwrap().get_class_instance(&name);

    // FIXME: Make explicit memory leak because current value is stored on the stack and we can't
    // make a policy of freeing results since it wont apply in all cases. It could be solved by a
    // reference table, but that does not work well with rust.
    let mut boxed = ManuallyDrop::new(Box::new(class));
    // forget(&boxed);
    let ptr = (&mut **boxed) as *mut Rc<UnsafeCell<Object>> as jclass;
    forget(ptr);
    ptr
}

pub unsafe extern "C" fn print_stream_hook(env: *mut JNIEnv, obj: jobject, fd: jint, str: jstring) {
    println!("Got print to {}", fd);
}

