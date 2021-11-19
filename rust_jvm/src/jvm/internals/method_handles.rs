#![allow(unused_variables)]
use crate::jvm::call::RawJNIEnv;
use crate::jvm::mem::{
    ClassHandle, FieldDescriptor, ManualInstanceReference, ObjArrayHandle, ObjectHandle,
    ObjectReference, StringHandle,
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
    name: Option<ObjArrayHandle>,
) -> jint {
    unimplemented!()
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
