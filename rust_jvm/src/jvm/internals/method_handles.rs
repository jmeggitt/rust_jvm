#![allow(unused_variables)]
use crate::jvm::call::RawJNIEnv;
use crate::jvm::mem::{
    ClassHandle, FieldDescriptor, ManualInstanceReference, ObjArrayHandle, ObjectHandle,
    ObjectReference, RawArrayObject, StringHandle,
};
use jni::sys::{jint, jlong, jobject};

pub unsafe extern "system" fn method_handle_natives_init(
    env: RawJNIEnv,
    _cls: Option<ClassHandle>,
    self_obj: Option<ObjectHandle>,
    reference: Option<ObjectHandle>,
) {
    unimplemented!()
}

pub unsafe extern "system" fn method_handle_natives_expand(
    env: RawJNIEnv,
    _cls: Option<ClassHandle>,
    self_obj: Option<ObjectHandle>,
) {
    unimplemented!()
}

pub unsafe extern "system" fn method_handle_natives_resolve(
    env: RawJNIEnv,
    _cls: Option<ClassHandle>,
    member_name: Option<ObjectHandle>,
    caller: Option<ClassHandle>,
) -> Option<ObjectHandle> {
    let instance = member_name?.expect_instance();
    let mut instance_lock = instance.lock();

    let parent_class: Option<ObjectHandle> = instance_lock.read_named_field("clazz");
    let field_name: Option<ObjectHandle> = instance_lock.read_named_field("name");
    let type_name: Option<ObjectHandle> = instance_lock.read_named_field("type");

    let parent_class = parent_class?.unwrap_as_class();
    let field_name = field_name?.expect_string();
    let desc = format!("{}", FieldDescriptor::from_class(type_name?));

    let lock = env.read();
    let raw_class = lock.class_loader.class(&parent_class)?;

    let access = if type_name?.get_class() == "java/lang/Class" {
        let field = raw_class
            .get_field(&field_name, &desc)
            .expect("Field not found");
        field.access
    } else {
        let method = raw_class
            .get_method(&field_name, &desc)
            .expect("Field not found");
        method.access
    };

    let flags: i32 = instance_lock.read_named_field("flags");
    instance_lock.write_named_field("flags", flags | (access.bits() as i32));
    member_name
}

pub unsafe extern "system" fn method_handle_natives_get_members(
    env: RawJNIEnv,
    _cls: Option<ClassHandle>,
    defc: Option<ClassHandle>,
    match_name: Option<StringHandle>,
    match_sig: Option<StringHandle>,
    match_flags: jint,
    caller: Option<ClassHandle>,
    skip: jint,
    results: Option<ObjArrayHandle>,
) -> jint {
    unimplemented!()
}

pub unsafe extern "system" fn method_handle_natives_object_field_offset(
    env: RawJNIEnv,
    _cls: Option<ClassHandle>,
    self_obj: Option<ObjectHandle>,
) -> jlong {
    unimplemented!()
}

pub unsafe extern "system" fn method_handle_natives_static_field_ffset(
    env: RawJNIEnv,
    _cls: Option<ClassHandle>,
    self_obj: Option<ObjectHandle>,
) -> jlong {
    unimplemented!()
}

pub unsafe extern "system" fn method_handle_natives_static_field_base(
    env: RawJNIEnv,
    _cls: Option<ClassHandle>,
    self_obj: Option<ObjectHandle>,
) -> jobject {
    unimplemented!()
}

pub unsafe extern "system" fn method_handle_natives_get_member_vm_info(
    env: RawJNIEnv,
    _cls: Option<ClassHandle>,
    self_obj: Option<ObjectHandle>,
) -> jobject {
    unimplemented!()
}

pub unsafe extern "system" fn method_handle_natives_get_constant(
    _env: RawJNIEnv,
    _cls: Option<ClassHandle>,
    _which: jint,
) -> jint {
    0 // idk what COMPILER2 is, but since I don't have it this is always 0
}

pub unsafe extern "system" fn method_handle_natives_set_call_site_target_normal(
    env: RawJNIEnv,
    _cls: Option<ClassHandle>,
    call_site: Option<ObjectHandle>,
    target: Option<ObjectHandle>,
) {
    unimplemented!()
}

pub unsafe extern "system" fn method_handle_natives_set_call_site_target_volatile(
    env: RawJNIEnv,
    _cls: Option<ClassHandle>,
    call_site: Option<ObjectHandle>,
    target: Option<ObjectHandle>,
) {
    unimplemented!()
}

pub unsafe extern "system" fn method_handle_natives_get_named_con(
    env: RawJNIEnv,
    _cls: Option<ClassHandle>,
    which: jint,
    name_box: Option<RawArrayObject<Option<ObjectHandle>>>,
) -> jint {
    // TODO: This is supposed to check that these fields match the values used in the class files
    let (res, name) = match which {
        0 => (4, "GC_COUNT_GWT"),
        1 => (5, "GC_LAMBDA_SUPPORT"),
        2 => (0x00010000, "MN_IS_METHOD"),
        3 => (0x00020000, "MN_IS_CONSTRUCTOR"),
        4 => (0x00040000, "MN_IS_FIELD"),
        5 => (0x00080000, "MN_IS_TYPE"),
        6 => (0x00100000, "MN_CALLER_SENSITIVE"),
        7 => (24, "MN_REFERENCE_KIND_SHIFT"),
        8 => (0xF, "MN_REFERENCE_KIND_MASK"),
        9 => (0x00100000, "MN_SEARCH_SUPERCLASSES"),
        10 => (0x00200000, "MN_SEARCH_INTERFACES"),
        11 => (4, "T_BOOLEAN"),
        12 => (5, "T_CHAR"),
        13 => (6, "T_FLOAT"),
        14 => (7, "T_DOUBLE"),
        15 => (8, "T_BYTE"),
        16 => (9, "T_SHORT"),
        17 => (10, "T_INT"),
        18 => (11, "T_LONG"),
        19 => (12, "T_OBJECT"),
        20 => (14, "T_VOID"),
        21 => (99, "T_ILLEGAL"),
        // TODO: May be expecting char and byte constants as well
        _ => return 0,
    };

    if let Some(array) = name_box {
        let name_string = env.write().build_string(name).expect_object();
        let mut lock = array.lock();
        lock[0] = Some(name_string);
    }

    res
}

pub unsafe extern "system" fn method_handle_invoke(
    env: RawJNIEnv,
    handle: Option<ObjectHandle>,
    args: Option<ObjArrayHandle>,
) {
    // idk why they even included it in the standard it they were not going to allow it.
    unimplemented!("This use of reflection is unsupported.")
}

pub unsafe extern "system" fn method_handle_invoke_exact(
    env: RawJNIEnv,
    handle: Option<ObjectHandle>,
    args: Option<ObjArrayHandle>,
) {
    // idk why they even included it in the standard it they were not going to allow it.
    unimplemented!("This use of reflection is unsupported.")
}
