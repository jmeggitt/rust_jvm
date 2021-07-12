//! TODO: Split into seperate crate for shared library object

use std::cell::UnsafeCell;
use std::collections::hash_map::DefaultHasher;
use std::ffi::c_void;
use std::hash::{Hash, Hasher};
use std::mem::transmute;
use std::rc::Rc;

use jni::sys::{
    jboolean, jbyte, jchar, jclass, jdouble, jfloat, jint, jlong, jobject, jshort, jstring, JNIEnv,
};
use walkdir::WalkDir;

use crate::jvm::interface::GLOBAL_JVM;
use crate::jvm::mem::{ConstTypeId, JavaPrimitive, LocalVariable, ObjectHandle, ObjectReference, ObjectType};
use crate::jvm::{clean_str, JVM};

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

    // jvm.init_class("java/lang/System");

    jvm.linked_libraries.register_fn(
        "java/lang/System",
        "arraycopy",
        "(Ljava/lang/Object;ILjava/lang/Object;II)V",
        array_copy as *const c_void,
    );

    // jvm.init_class("java/lang/Class");

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
    jvm.init_class("jvm/hooks/PrintStreamHook");
    let stdout = jvm
        .exec_static(
            "jvm/hooks/PrintStreamHook",
            "buildStream",
            "(I)Ljava/io/PrintStream;",
            vec![LocalVariable::Int(0)],
        )
        .unwrap()
        .unwrap();
    // jvm.locals[0] = LocalVariable::Int(1);
    let stderr = jvm
        .exec_static(
            "jvm/hooks/PrintStreamHook",
            "buildStream",
            "(I)Ljava/io/PrintStream;",
            vec![LocalVariable::Int(1)],
        )
        .unwrap()
        .unwrap();

    let field_reference = format!("{}_{}", clean_str("java/lang/System"), clean_str("out"));
    jvm.static_fields.insert(field_reference, stdout);

    let field_reference = format!("{}_{}", clean_str("java/lang/System"), clean_str("err"));
    jvm.static_fields.insert(field_reference, stderr);
}

pub unsafe extern "system" fn hash_object(_env: *mut JNIEnv, obj: jobject) -> jint {
    let mut hasher = DefaultHasher::new();
    ObjectHandle::from_ptr(obj).unwrap().hash(&mut hasher);
    hasher.finish() as jint
    // let name_object = &*(obj as *const Rc<UnsafeCell<Object>>);
    // (&*name_object.get()).hash(&mut hasher);
    // let result = hasher.finish();
    // let [a, b] = transmute::<_, [i32; 2]>(result);
    // a ^ b
}

pub unsafe extern "system" fn array_copy(
    _env: *mut JNIEnv,
    cls: jclass,
    src: jobject,
    src_pos: jint,
    dst: jobject,
    dst_pos: jint,
    length: jint,
) {
    debug!(
        "Got correct version of arraycopy with src: {:p} dst: {:p}",
        src, dst
    );

    let src_object = ObjectHandle::from_ptr(src).unwrap();
    let dst_object = ObjectHandle::from_ptr(dst).unwrap();

    if src_object.memory_layout() != dst_object.memory_layout() {
        panic!("Attempted arraycopy with different typed arrays!");
    }

    match src_object.memory_layout() {
        ObjectType::Array(jboolean::ID) => src_object.expect_array::<jboolean>().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(jbyte::ID) => src_object.expect_array::<jbyte>().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(jchar::ID) => src_object.expect_array::<jchar>().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(jshort::ID) => src_object.expect_array::<jshort>().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(jint::ID) => src_object.expect_array::<jint>().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(jlong::ID) => src_object.expect_array::<jlong>().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(jfloat::ID) => src_object.expect_array::<jfloat>().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(jdouble::ID) => src_object.expect_array::<jdouble>().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(<Option<ObjectHandle>>::ID) => src_object
            .expect_array::<Option<ObjectHandle>>()
            .array_copy(
                dst_object,
                src_pos as usize,
                dst_pos as usize,
                length as usize,
            ),
        x => panic!("Array copy can not be preformed with type {:?}", x),
    };

    // let cls_object = &*(cls as *const Rc<UnsafeCell<Object>>);
    // debug!("Got class object: {:?}", cls_object);
    //
    // let src_object = &*(&*(src as *const Rc<UnsafeCell<Object>>)).get();
    // let dst_object = &mut *(&*(dst as *const Rc<UnsafeCell<Object>>)).get();
    //
    // let src_vec = match &*src_object {
    //     Object::Array { values, .. } => values,
    //     x => panic!(
    //         "Attempted to call arraycopy with non array entries: {:?}",
    //         x
    //     ),
    // };
    //
    // let dst_vec = match &mut *dst_object {
    //     Object::Array { values, .. } => values,
    //     x => panic!(
    //         "Attempted to call arraycopy with non array entries: {:?}",
    //         x
    //     ),
    // };
    //
    // // Be lazy since we need to clone each element
    // for offset in 0..length as usize {
    //     dst_vec[dst_pos as usize + offset] = src_vec[src_pos as usize + offset].clone();
    // }
}

pub unsafe extern "system" fn empty(_env: *mut JNIEnv, _cls: jclass) {}

pub unsafe extern "system" fn desired_assertions(
    _env: *mut JNIEnv,
    _cls: jclass,
    _target: jclass,
) -> jboolean {
    0 // Don't do assertions, I don't need the extra work
}

pub unsafe extern "system" fn get_class(_env: *mut JNIEnv, _cls: jclass, name: jstring) -> jclass {
    debug!("Executing getPrimitiveClass");
    // TODO: use call to JNIEnv to read string
    let name_object = ObjectHandle::from_ptr(name);
    let name_string = name_object.unwrap().expect_string();

    // let name_object = &*(name as *const Rc<UnsafeCell<Object>>);
    // debug!("Received object: {:p}", name);
    // debug!("Object: {:p}, debug: {:?}", name, name_object);

    // let name = (&*name_object.get()).expect_string();
    // info!("Getting class named {:?}", name);

    let class = GLOBAL_JVM
        .as_mut()
        .unwrap()
        .get_class_instance(&name_string);

    // FIXME: Make explicit memory leak because current value is stored on the stack and we can't
    // make a policy of freeing results since it wont apply in all cases. It could be solved by a
    // reference table, but that does not work well with rust.
    // Box::leak(Box::new(class)) as *mut Rc<UnsafeCell<Object>> as jclass
    class.unwrap_unknown().into_raw()
}

pub unsafe extern "C" fn print_stream_hook(
    _env: *mut JNIEnv,
    _obj: jobject,
    fd: jint,
    _str: jstring,
) {
    panic!("Got print to {}", fd);
}
