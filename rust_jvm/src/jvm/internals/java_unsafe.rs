use crate::class::BufferedRead;
use crate::jvm::call::{JavaEnvInvoke, RawJNIEnv};
use crate::jvm::mem::{
    ArrayReference, FieldDescriptor, InstanceReference, JavaPrimitive, JavaTypeEnum, JavaValue,
    ManualInstanceReference, ObjectHandle, ObjectReference, ObjectType,
};
use crate::jvm::thread::SynchronousMonitor;
use jni::sys::{
    jboolean, jbyte, jbyteArray, jchar, jclass, jdouble, jdoubleArray, jfloat, jint, jlong,
    jobject, jobjectArray, jshort, jstring, jthrowable, jvalue, JNI_FALSE,
};
use libc::{free, malloc, realloc};
use std::ffi::c_void;
use std::mem::{size_of, transmute};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicI32, AtomicI64, AtomicPtr, Ordering};

// TODO: Fill in unsafe

/// Class:     sun_misc_Unsafe
/// Method:    getInt
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getInt__Ljava_lang_Object_2J(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jint {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putInt
/// Signature: (Ljava/lang/Object{ unimplemented!() }JI)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putInt__Ljava_lang_Object_2JI(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jint,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getObject
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)Ljava/lang/Object{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getObject(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jobject {
    assert_eq!(size_of::<jobject>(), size_of::<Option<ObjectHandle>>());
    *obj_expect!(env, obj, null_mut()).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putObject
/// Signature: (Ljava/lang/Object{ unimplemented!() }JLjava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putObject(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jobject,
) {
    assert_eq!(size_of::<jobject>(), size_of::<Option<ObjectHandle>>());
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getBoolean
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getBoolean(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jboolean {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putBoolean
/// Signature: (Ljava/lang/Object{ unimplemented!() }JZ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putBoolean(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jboolean,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getByte
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)B
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getByte__Ljava_lang_Object_2J(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jbyte {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putByte
/// Signature: (Ljava/lang/Object{ unimplemented!() }JB)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putByte__Ljava_lang_Object_2JB(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jbyte,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getShort
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)S
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getShort__Ljava_lang_Object_2J(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jshort {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putShort
/// Signature: (Ljava/lang/Object{ unimplemented!() }JS)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putShort__Ljava_lang_Object_2JS(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jshort,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getChar
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)C
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getChar__Ljava_lang_Object_2J(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jchar {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putChar
/// Signature: (Ljava/lang/Object{ unimplemented!() }JC)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putChar__Ljava_lang_Object_2JC(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jchar,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getLong
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getLong__Ljava_lang_Object_2J(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jlong {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putLong
/// Signature: (Ljava/lang/Object{ unimplemented!() }JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putLong__Ljava_lang_Object_2JJ(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jlong,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getFloat
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)F
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getFloat__Ljava_lang_Object_2J(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jfloat {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putFloat
/// Signature: (Ljava/lang/Object{ unimplemented!() }JF)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putFloat__Ljava_lang_Object_2JF(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jfloat,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getDouble
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)D
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getDouble__Ljava_lang_Object_2J(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jdouble {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putDouble
/// Signature: (Ljava/lang/Object{ unimplemented!() }JD)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putDouble__Ljava_lang_Object_2JD(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jdouble,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getByte
/// Signature: (J)B
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getByte__J(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
) -> jbyte {
    let ptr: *mut _ = transmute(offset as isize);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putByte
/// Signature: (JB)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putByte__JB(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
    val: jbyte,
) {
    let ptr: *mut _ = transmute(offset as isize);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getShort
/// Signature: (J)S
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getShort__J(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
) -> jshort {
    let ptr: *mut _ = transmute(offset as isize);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putShort
/// Signature: (JS)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putShort__JS(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
    val: jshort,
) {
    let ptr: *mut _ = transmute(offset as isize);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getChar
/// Signature: (J)C
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getChar__J(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
) -> jchar {
    let ptr: *mut _ = transmute(offset as isize);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putChar
/// Signature: (JC)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putChar__JC(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
    val: jchar,
) {
    let ptr: *mut _ = transmute(offset as isize);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getInt
/// Signature: (J)I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getInt__J(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
) -> jint {
    let ptr: *mut _ = transmute(offset as isize);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putInt
/// Signature: (JI)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putInt__JI(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
    val: jint,
) {
    let ptr: *mut _ = transmute(offset as isize);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getLong
/// Signature: (J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getLong__J(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
) -> jlong {
    let ptr: *mut jlong = transmute(offset as isize);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putLong
/// Signature: (JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putLong__JJ(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
    val: jlong,
) {
    let ptr: *mut jlong = transmute(offset as isize);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getFloat
/// Signature: (J)F
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getFloat__J(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
) -> jfloat {
    let ptr: *mut _ = transmute(offset as isize);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putFloat
/// Signature: (JF)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putFloat__JF(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
    val: jfloat,
) {
    let ptr: *mut _ = transmute(offset as isize);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getDouble
/// Signature: (J)D
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getDouble__J(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
) -> jdouble {
    let ptr: *mut _ = transmute(offset as isize);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putDouble
/// Signature: (JD)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putDouble__JD(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
    val: jdouble,
) {
    let ptr: *mut _ = transmute(offset as isize);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getAddress
/// Signature: (J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getAddress(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
) -> jlong {
    // TODO: This will fail to compile on non-x64 machines
    let ptr: *mut _ = transmute(offset as isize);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putAddress
/// Signature: (JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putAddress(
    _env: RawJNIEnv,
    _this: jobject,
    offset: jlong,
    val: jlong,
) {
    let ptr: *mut _ = transmute(offset as isize);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    allocateMemory
/// Signature: (J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_allocateMemory(
    _env: RawJNIEnv,
    _this: jobject,
    size: jlong,
) -> jlong {
    let ret = malloc(size as _) as i64;
    debug!("Malloced: 0x{:X} ({} bytes)", ret, size);
    ret
}

/// Class:     sun_misc_Unsafe
/// Method:    reallocateMemory
/// Signature: (JJ)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_reallocateMemory(
    _env: RawJNIEnv,
    _this: jobject,
    ptr: jlong,
    new_size: jlong,
) -> jlong {
    let ret = realloc(transmute(ptr as isize), new_size as _) as i64;
    debug!("Reallocating: 0x{:X} -> 0x{:X}", ptr, ret);
    ret
}

/// Class:     sun_misc_Unsafe
/// Method:    setMemory
/// Signature: (Ljava/lang/Object;JJB)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_setMemory(
    _env: RawJNIEnv,
    _this: jobject,
    _obj: jobject,
    _offset: jlong,
    _valb: jlong,
    _: jbyte,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    copyMemory
/// Signature: (Ljava/lang/Object;JLjava/lang/Object;JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_copyMemory(
    _env: RawJNIEnv,
    _this: jobject,
    _obj: jobject,
    _offset: jlong,
    _objb: jobject,
    _valb: jlong,
    _valc: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    freeMemory
/// Signature: (J)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_freeMemory(
    _env: RawJNIEnv,
    _this: jobject,
    ptr: jlong,
) {
    debug!("Freeing pointer: 0x{:X}", ptr);
    free(transmute(ptr as isize))
}

/// Class:     sun_misc_Unsafe
/// Method:    staticFieldOffset
/// Signature: (Ljava/lang/reflect/Field{ unimplemented!() })J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_staticFieldOffset(
    env: RawJNIEnv,
    _this: jobject,
    field: jobject,
) -> jlong {
    let field = obj_expect!(env, field, 0);
    let instance = field.expect_instance();

    let class: Option<ObjectHandle> = instance.read_named_field("clazz");
    let class_name: Option<ObjectHandle> =
        class.unwrap().expect_instance().read_named_field("name");
    let class_name = class_name.unwrap().expect_string().replace('.', "/");
    let field_name: Option<ObjectHandle> = instance.read_named_field("name");
    let field_name = field_name.unwrap().expect_string();

    // TODO: Should we allocate a slot if none exists?
    let lock = env.read();
    match lock
        .static_fields
        .get_field_offset(&class_name, &field_name)
    {
        Some(v) => v as jlong,
        None => -1,
    }
}

/// Class:     sun_misc_Unsafe
/// Method:    objectFieldOffset
/// Signature: (Ljava/lang/reflect/Field;)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_objectFieldOffset(
    env: RawJNIEnv,
    _this: jobject,
    field: jobject,
) -> jlong {
    let field = obj_expect!(env, field, 0);
    let instance = field.expect_instance();

    let class: Option<ObjectHandle> = instance.read_named_field("clazz");
    let class_name: Option<ObjectHandle> =
        class.unwrap().expect_instance().read_named_field("name");
    let class_name = class_name.unwrap().expect_string().replace('.', "/");
    let field_name: Option<ObjectHandle> = instance.read_named_field("name");

    let mut lock = env.write();
    let schema = lock.class_schema(&class_name);

    schema
        .field_offsets
        .get(&field_name.unwrap().expect_string())
        .unwrap()
        .offset as jlong
}

/// Class:     sun_misc_Unsafe
/// Method:    staticFieldBase
/// Signature: (Ljava/lang/reflect/Field{ unimplemented!() })Ljava/lang/Object{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_staticFieldBase(
    env: RawJNIEnv,
    _this: jobject,
    _obj: jobject,
) -> jobject {
    // TODO: Having separate static objects for each class would probably be more efficient.
    env.read().static_fields.static_obj.ptr()
}

/// Class:     sun_misc_Unsafe
/// Method:    shouldBeInitialized
/// Signature: (Ljava/lang/Class;)Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_shouldBeInitialized(
    env: RawJNIEnv,
    _this: jobject,
    class: jclass,
) -> jboolean {
    let name = obj_expect!(env, class, JNI_FALSE).unwrap_as_class();
    env.read().static_load.contains(&name) as jboolean
}

/// Class:     sun_misc_Unsafe
/// Method:    ensureClassInitialized
/// Signature: (Ljava/lang/Class{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_ensureClassInitialized(
    mut env: RawJNIEnv,
    _this: jobject,
    class: jclass,
) {
    let class_obj = obj_expect!(env, class);
    let name = class_obj.unwrap_as_class();
    env.init_class(&name);
}

/// Class:     sun_misc_Unsafe
/// Method:    arrayBaseOffset
/// Signature: (Ljava/lang/Class{ unimplemented!() })I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_arrayBaseOffset(
    _env: RawJNIEnv,
    _this: jobject,
    _: jclass,
) -> jint {
    0
}

/// Class:     sun_misc_Unsafe
/// Method:    arrayIndexScale
/// Signature: (Ljava/lang/Class{ unimplemented!() })I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_arrayIndexScale(
    _env: RawJNIEnv,
    _this: jobject,
    target: jclass,
) -> jint {
    let a = ObjectHandle::from_ptr(target).unwrap().expect_instance();
    let name_obj: Option<ObjectHandle> = a.read_named_field("name");
    let name = name_obj.unwrap().expect_string().replace('.', "/");
    if let Ok(FieldDescriptor::Array(arr)) = FieldDescriptor::read_str(&name) {
        match &*arr {
            FieldDescriptor::Byte => return size_of::<jbyte>() as i32,
            FieldDescriptor::Char => return size_of::<jchar>() as i32,
            FieldDescriptor::Double => return size_of::<jdouble>() as i32,
            FieldDescriptor::Float => return size_of::<jfloat>() as i32,
            FieldDescriptor::Int => return size_of::<jint>() as i32,
            FieldDescriptor::Long => return size_of::<jlong>() as i32,
            FieldDescriptor::Short => return size_of::<jshort>() as i32,
            FieldDescriptor::Boolean => return size_of::<jboolean>() as i32,
            FieldDescriptor::Object(_) => return size_of::<Option<ObjectHandle>>() as i32,
            FieldDescriptor::Array(_) => return size_of::<Option<ObjectHandle>>() as i32,
            FieldDescriptor::Void => return size_of::<()>() as i32,
            FieldDescriptor::Method { .. } => return size_of::<usize>() as i32,
        }
    }

    panic!("Not an array class!")
}

/// Class:     sun_misc_Unsafe
/// Method:    addressSize
/// Signature: ()I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_addressSize(
    _env: RawJNIEnv,
    _this: jobject,
) -> jint {
    size_of::<*const c_void>() as jint
}

/// Class:     sun_misc_Unsafe
/// Method:    pageSize
/// Signature: ()I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_pageSize(
    _env: RawJNIEnv,
    _this: jobject,
) -> jint {
    page_size::get() as jint
}

/// Class:     sun_misc_Unsafe
/// Method:    defineClass
/// Signature: (Ljava/lang/String;[BIILjava/lang/ClassLoader;Ljava/security/ProtectionDomain;)Ljava/lang/Class;
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_defineClass(
    _env: RawJNIEnv,
    _this: jobject,
    _: jstring,
    _: jbyteArray,
    _val: jint,
    _valb: jint,
    _objc: jobject,
    _obj: jobject,
) {
    unimplemented!("Runtime defined classes are not supported")
}

/// Class:     sun_misc_Unsafe
/// Method:    defineAnonymousClass
/// Signature: (Ljava/lang/Class{ unimplemented!() }[B[Ljava/lang/Object{ unimplemented!() })Ljava/lang/Class{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_defineAnonymousClass(
    _env: RawJNIEnv,
    _this: jobject,
    _: jclass,
    _: jbyteArray,
    _obj: jobjectArray,
) {
    unimplemented!("Anonymous classes are not currently supported")
}

/// Class:     sun_misc_Unsafe
/// Method:    allocateInstance
/// Signature: (Ljava/lang/Class{ unimplemented!() })Ljava/lang/Object{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_allocateInstance(
    env: RawJNIEnv,
    _this: jobject,
    class: jclass,
) -> jobject {
    let name = obj_expect!(env, class, null_mut()).unwrap_as_class();
    ObjectHandle::new(env.write().class_schema(&name)).ptr()
}

/// Class:     sun_misc_Unsafe
/// Method:    monitorEnter
/// Signature: (Ljava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_monitorEnter(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
) {
    env.lock(obj_expect!(env, obj));
}

/// Class:     sun_misc_Unsafe
/// Method:    monitorExit
/// Signature: (Ljava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_monitorExit(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
) {
    env.unlock(obj_expect!(env, obj));
}

/// Class:     sun_misc_Unsafe
/// Method:    tryMonitorEnter
/// Signature: (Ljava/lang/Object{ unimplemented!() })Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_tryMonitorEnter(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
) -> jboolean {
    env.try_lock(obj_expect!(env, obj)) as jboolean
}

/// Class:     sun_misc_Unsafe
/// Method:    throwException
/// Signature: (Ljava/lang/Throwable{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_throwException(
    env: RawJNIEnv,
    _this: jobject,
    obj: jthrowable,
) {
    env.write_thrown(Some(obj_expect!(env, obj)))
}

/// Class:     sun_misc_Unsafe
/// Method:    compareAndSwapObject
/// Signature: (Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapObject(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    expected: jobject,
    x: jobject,
) -> jboolean {
    let obj = obj_expect!(env, obj, JNI_FALSE);

    match obj.memory_layout() {
        ObjectType::Instance => {
            let instance = obj.expect_instance();
            assert_eq!(offset as usize % size_of::<jvalue>(), 0);
            let index = offset as usize / size_of::<jvalue>();

            let fields = instance.raw_fields();
            assert!(index < fields.len());

            let ptr = &mut fields[index] as *mut jvalue as *const AtomicPtr<_>;

            let res = (&*ptr).compare_exchange(expected, x, Ordering::SeqCst, Ordering::Relaxed);
            res.is_ok() as jboolean
        }
        ObjectType::Array(JavaTypeEnum::Reference) => {
            let instance = obj.expect_array::<Option<ObjectHandle>>();
            assert_eq!(offset as usize % size_of::<Option<ObjectHandle>>(), 0);
            let index = offset as usize / size_of::<Option<ObjectHandle>>();

            let fields = instance.raw_fields();
            assert!(index < fields.len());

            // TODO: I feel like this could lead to a segfault, check pointer size just to be safe
            assert_eq!(size_of::<Option<ObjectHandle>>(), size_of::<jobject>());
            let ptr = &mut fields[index] as *mut Option<ObjectHandle> as *const AtomicPtr<_>;

            let res = (&*ptr).compare_exchange(expected, x, Ordering::SeqCst, Ordering::Relaxed);
            res.is_ok() as jboolean
        }
        _ => panic!(),
    }
}

/// Class:     sun_misc_Unsafe
/// Method:    compareAndSwapInt
/// Signature: (Ljava/lang/Object{ unimplemented!() }JII)Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapInt(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    expected: jint,
    x: jint,
) -> jboolean {
    let obj = obj_expect!(env, obj, JNI_FALSE);
    let instance = obj.expect_instance();
    assert_eq!(offset as usize % size_of::<jvalue>(), 0);
    let index = offset as usize / size_of::<jvalue>();

    let fields = instance.raw_fields();
    assert!(index < fields.len());

    let ptr = &mut fields[index] as *mut jvalue as *const AtomicI32;

    let res = (&*ptr).compare_exchange(expected, x, Ordering::SeqCst, Ordering::Relaxed);
    res.is_ok() as jboolean
}

/// Class:     sun_misc_Unsafe
/// Method:    compareAndSwapLong
/// Signature: (Ljava/lang/Object{ unimplemented!() }JJJ)Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapLong(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    expected: jlong,
    x: jlong,
) -> jboolean {
    let obj = obj_expect!(env, obj, JNI_FALSE);
    let instance = obj.expect_instance();
    assert_eq!(offset as usize % size_of::<jvalue>(), 0);
    let index = offset as usize / size_of::<jvalue>();

    let fields = instance.raw_fields();
    assert!(index < fields.len());

    let ptr = &mut fields[index] as *mut jvalue as *const AtomicI64;

    let res = (&*ptr).compare_exchange(expected, x, Ordering::SeqCst, Ordering::Relaxed);
    res.is_ok() as jboolean
}

/// Class:     sun_misc_Unsafe
/// Method:    getObjectVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)Ljava/lang/Object{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getObjectVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jobject {
    let object = obj_expect!(env, obj, null_mut());

    {
        let lock = env.read();
        if lock.static_fields.static_obj == object {
            if let JavaValue::Reference(v) = lock.static_fields[offset as usize] {
                return v.pack().l;
            } else {
                todo!("Throw IllegalArgumentException")
            }
        }
    }

    match object.memory_layout() {
        ObjectType::Instance => {
            let ret: Option<ObjectHandle> = object.expect_instance().read_field(offset as usize);
            ret.pack().l
        }
        ObjectType::Array(JavaTypeEnum::Reference) => {
            // let scale = Java_sun_misc_Unsafe_arrayIndexScale(env, _this, obj);
            let ret: Option<ObjectHandle> = object
                .expect_array::<Option<ObjectHandle>>()
                .read_array(offset as usize / size_of::<Option<ObjectHandle>>());
            ret.pack().l
        }
        _ => panic!(),
    }
}

/// Class:     sun_misc_Unsafe
/// Method:    putObjectVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JLjava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putObjectVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jobject,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getIntVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getIntVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jint {
    let object = obj_expect!(env, obj, 0);

    {
        let lock = env.read();
        if lock.static_fields.static_obj == object {
            if let JavaValue::Int(v) = lock.static_fields[offset as usize] {
                return v;
            } else {
                todo!("Throw IllegalArgumentException")
            }
        }
    }

    object.expect_instance().read_field(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putIntVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JI)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putIntVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jint,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getBooleanVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getBooleanVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jboolean {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putBooleanVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JZ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putBooleanVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jboolean,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getByteVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)B
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getByteVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jbyte {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putByteVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JB)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putByteVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jbyte,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getShortVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)S
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getShortVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jshort {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putShortVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JS)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putShortVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jshort,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getCharVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)C
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getCharVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jchar {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putCharVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JC)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putCharVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jchar,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getLongVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getLongVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jlong {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putLongVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putLongVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jlong,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getFloatVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)F
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getFloatVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jfloat {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putFloatVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JF)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putFloatVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jfloat,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getDoubleVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)D
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getDoubleVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
) -> jdouble {
    *obj_expect!(env, obj).raw_object_memory(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putDoubleVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JD)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putDoubleVolatile(
    env: RawJNIEnv,
    _this: jobject,
    obj: jobject,
    offset: jlong,
    val: jdouble,
) {
    *obj_expect!(env, obj).raw_object_memory(offset as usize) = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    putOrderedObject
/// Signature: (Ljava/lang/Object{ unimplemented!() }JLjava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putOrderedObject(
    _env: RawJNIEnv,
    _this: jobject,
    _obj: jobject,
    _offset: jlong,
    _objb: jobject,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putOrderedInt
/// Signature: (Ljava/lang/Object{ unimplemented!() }JI)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putOrderedInt(
    _env: RawJNIEnv,
    _this: jobject,
    _obj: jobject,
    _offset: jlong,
    _val: jint,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putOrderedLong
/// Signature: (Ljava/lang/Object{ unimplemented!() }JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putOrderedLong(
    _env: RawJNIEnv,
    _this: jobject,
    _obj: jobject,
    _offset: jlong,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    unpark
/// Signature: (Ljava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_unpark(
    _env: RawJNIEnv,
    _this: jobject,
    _obj: jobject,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    park
/// Signature: (ZJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_park(
    _env: RawJNIEnv,
    _this: jobject,
    _: jboolean,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getLoadAverage
/// Signature: ([DI)I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getLoadAverage(
    _env: RawJNIEnv,
    _this: jobject,
    _: jdoubleArray,
    _val: jint,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    loadFence
/// Signature: ()V
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_loadFence(_env: RawJNIEnv, _this: jobject) {
    std::sync::atomic::fence(Ordering::Acquire);
}

/// Class:     sun_misc_Unsafe
/// Method:    storeFence
/// Signature: ()V
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_storeFence(_env: RawJNIEnv, _this: jobject) {
    std::sync::atomic::fence(Ordering::Release);
}

/// Class:     sun_misc_Unsafe
/// Method:    fullFence
/// Signature: ()V
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_fullFence(_env: RawJNIEnv, _this: jobject) {
    std::sync::atomic::fence(Ordering::SeqCst);
}
