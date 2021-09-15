use crate::class::BufferedRead;
use crate::jvm::call::RawJNIEnv;
use crate::jvm::mem::{FieldDescriptor, InstanceReference, ManualInstanceReference, ObjectHandle};
use jni::objects::JObject;
use jni::sys::{
    jboolean, jbyte, jbyteArray, jchar, jclass, jdouble, jdoubleArray, jfloat, jint, jlong,
    jobject, jobjectArray, jshort, jstring, jthrowable, jvalue, JNI_FALSE,
};
use jni::JNIEnv;
use std::mem::{size_of, transmute};
use std::sync::atomic::{AtomicI32, AtomicPtr, Ordering};
use libc::{malloc, realloc, free};

// TODO: Fill in unsafe

/// Class:     sun_misc_Unsafe
/// Method:    getInt
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getInt__Ljava_lang_Object_2J(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putInt
/// Signature: (Ljava/lang/Object{ unimplemented!() }JI)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putInt__Ljava_lang_Object_2JI(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _offset: jlong,
    _val: jint,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getObject
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)Ljava/lang/Object{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getObject(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putObject
/// Signature: (Ljava/lang/Object{ unimplemented!() }JLjava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putObject(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _ob: JObject,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getBoolean
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getBoolean(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putBoolean
/// Signature: (Ljava/lang/Object{ unimplemented!() }JZ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putBoolean(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _offset: jlong,
    _val: jboolean,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getByte
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)B
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getByte__Ljava_lang_Object_2J(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putByte
/// Signature: (Ljava/lang/Object{ unimplemented!() }JB)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putByte__Ljava_lang_Object_2JB(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _: jbyte,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getShort
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)S
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getShort__Ljava_lang_Object_2J(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putShort
/// Signature: (Ljava/lang/Object{ unimplemented!() }JS)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putShort__Ljava_lang_Object_2JS(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _: jshort,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getChar
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)C
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getChar__Ljava_lang_Object_2J(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putChar
/// Signature: (Ljava/lang/Object{ unimplemented!() }JC)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putChar__Ljava_lang_Object_2JC(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _: jchar,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getLong
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getLong__Ljava_lang_Object_2J(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putLong
/// Signature: (Ljava/lang/Object{ unimplemented!() }JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putLong__Ljava_lang_Object_2JJ(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    __val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getFloat
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)F
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getFloat__Ljava_lang_Object_2J(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putFloat
/// Signature: (Ljava/lang/Object{ unimplemented!() }JF)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putFloat__Ljava_lang_Object_2JF(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _: jfloat,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getDouble
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)D
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getDouble__Ljava_lang_Object_2J(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putDouble
/// Signature: (Ljava/lang/Object{ unimplemented!() }JD)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putDouble__Ljava_lang_Object_2JD(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _: jdouble,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getByte
/// Signature: (J)B
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getByte__J(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
) -> jbyte {
    let ptr: *mut _ = transmute(offset);
    debug!("Got byte: 0x{:X} -> 0x{:X}", offset, *ptr);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putByte
/// Signature: (JB)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putByte__JB(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
    val: jbyte,
) {
    let ptr: *mut _ = transmute(offset);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getShort
/// Signature: (J)S
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getShort__J(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
) -> jshort {
    let ptr: *mut _ = transmute(offset);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putShort
/// Signature: (JS)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putShort__JS(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
    val: jshort,
) {
    let ptr: *mut _ = transmute(offset);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getChar
/// Signature: (J)C
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getChar__J(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
) -> jchar {
    let ptr: *mut _ = transmute(offset);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putChar
/// Signature: (JC)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putChar__JC(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
    val: jchar,
) {
    let ptr: *mut _ = transmute(offset);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getInt
/// Signature: (J)I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getInt__J(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
) ->jint {
    let ptr: *mut _ = transmute(offset);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putInt
/// Signature: (JI)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putInt__JI(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
    val: jint,
) {
    let ptr: *mut _ = transmute(offset);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getLong
/// Signature: (J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getLong__J(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
) -> jlong {
    let ptr: *mut jlong = transmute(offset);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putLong
/// Signature: (JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putLong__JJ(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
    val: jlong,
) {
    debug!("Put long: 0x{:X} -> 0x{:X}", val, offset);
    let ptr: *mut jlong = transmute(offset);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getFloat
/// Signature: (J)F
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getFloat__J(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
) -> jfloat {
    let ptr: *mut _ = transmute(offset);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putFloat
/// Signature: (JF)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putFloat__JF(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
    val: jfloat,
) {
    let ptr: *mut _ = transmute(offset);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getDouble
/// Signature: (J)D
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getDouble__J(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
) -> jdouble {
    let ptr: *mut _ = transmute(offset);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putDouble
/// Signature: (JD)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putDouble__JD(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
    val: jdouble,
) {
    let ptr: *mut _ = transmute(offset);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    getAddress
/// Signature: (J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getAddress(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
) -> jlong {
    // TODO: This will fail to compile on non-x64 machines
    let ptr: *mut _ = transmute(offset);
    *ptr
}

/// Class:     sun_misc_Unsafe
/// Method:    putAddress
/// Signature: (JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putAddress(
    _env: JNIEnv,
    _this: JObject,
    offset: jlong,
    val: jlong,
) {
    let ptr: *mut _ = transmute(offset);
    *ptr = val;
}

/// Class:     sun_misc_Unsafe
/// Method:    allocateMemory
/// Signature: (J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_allocateMemory(
    _env: JNIEnv,
    _this: JObject,
    size: jlong,
) -> jlong {
    let ret = transmute(malloc(size as _));
    debug!("Malloced: 0x{:X} ({} bytes)", ret, size);
    ret
}

/// Class:     sun_misc_Unsafe
/// Method:    reallocateMemory
/// Signature: (JJ)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_reallocateMemory(
    _env: JNIEnv,
    _this: JObject,
    ptr: jlong,
    new_size: jlong,
) -> jlong {
    let ret = transmute(realloc(transmute(ptr), new_size as _));
    debug!("Reallocating: 0x{:X} -> 0x{:X}", ptr, ret);
    ret
}

/// Class:     sun_misc_Unsafe
/// Method:    setMemory
/// Signature: (Ljava/lang/Object{ unimplemented!() }JJB)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_setMemory(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _valb: jlong,
    _: jbyte,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    copyMemory
/// Signature: (Ljava/lang/Object{ unimplemented!() }JLjava/lang/Object{ unimplemented!() }JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_copyMemory(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _objb: JObject,
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
    _env: JNIEnv,
    _this: JObject,
    ptr: jlong,
) {
    debug!("Freeing pointer: 0x{:X}", ptr);
    free(transmute(ptr))
}

/// Class:     sun_misc_Unsafe
/// Method:    staticFieldOffset
/// Signature: (Ljava/lang/reflect/Field{ unimplemented!() })J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_staticFieldOffset(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    objectFieldOffset
/// Signature: (Ljava/lang/reflect/Field{ unimplemented!() })J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_objectFieldOffset(
    env: RawJNIEnv,
    _this: JObject,
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
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    shouldBeInitialized
/// Signature: (Ljava/lang/Class{ unimplemented!() })Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_shouldBeInitialized(
    _env: JNIEnv,
    _this: JObject,
    _: jclass,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    ensureClassInitialized
/// Signature: (Ljava/lang/Class{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_ensureClassInitialized(
    _env: JNIEnv,
    _this: JObject,
    _: jclass,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    arrayBaseOffset
/// Signature: (Ljava/lang/Class{ unimplemented!() })I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_arrayBaseOffset(
    _env: JNIEnv,
    _this: JObject,
    _: jclass,
) -> jint {
    0
}

/// Class:     sun_misc_Unsafe
/// Method:    arrayIndexScale
/// Signature: (Ljava/lang/Class{ unimplemented!() })I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_arrayIndexScale(
    _env: JNIEnv,
    _this: JObject,
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
    _env: JNIEnv,
    _this: JObject,
) -> jint {
    size_of::<usize>() as jint
}

/// Class:     sun_misc_Unsafe
/// Method:    pageSize
/// Signature: ()I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_pageSize(_env: JNIEnv, _this: JObject) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    defineClass
/// Signature: (Ljava/lang/String{ unimplemented!() }[BIILjava/lang/ClassLoader{ unimplemented!() }Ljava/security/ProtectionDomain{ unimplemented!() })Ljava/lang/Class{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_defineClass(
    _env: JNIEnv,
    _this: JObject,
    _: jstring,
    _: jbyteArray,
    _val: jint,
    _valb: jint,
    _objc: JObject,
    _obj: JObject,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    defineAnonymousClass
/// Signature: (Ljava/lang/Class{ unimplemented!() }[B[Ljava/lang/Object{ unimplemented!() })Ljava/lang/Class{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_defineAnonymousClass(
    _env: JNIEnv,
    _this: JObject,
    _: jclass,
    _: jbyteArray,
    _obj: jobjectArray,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    allocateInstance
/// Signature: (Ljava/lang/Class{ unimplemented!() })Ljava/lang/Object{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_allocateInstance(
    _env: JNIEnv,
    _this: JObject,
    _: jclass,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    monitorEnter
/// Signature: (Ljava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_monitorEnter(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    monitorExit
/// Signature: (Ljava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_monitorExit(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    tryMonitorEnter
/// Signature: (Ljava/lang/Object{ unimplemented!() })Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_tryMonitorEnter(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    throwException
/// Signature: (Ljava/lang/Throwable{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_throwException(
    _env: JNIEnv,
    _this: JObject,
    _: jthrowable,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    compareAndSwapObject
/// Signature: (Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapObject(
    env: RawJNIEnv,
    _this: JObject,
    obj: jobject,
    offset: jlong,
    expected: jobject,
    x: jobject,
) -> jboolean {
    let obj = obj_expect!(env, obj, JNI_FALSE);
    let instance = obj.expect_instance();
    assert_eq!(offset as usize % size_of::<jvalue>(), 0);
    let index = offset as usize / size_of::<jvalue>();

    let mut fields = instance.raw_fields();
    assert!(index < fields.len());

    let ptr = &mut fields[index] as *mut jvalue as *const AtomicPtr<_>;

    let res = (&*ptr).compare_exchange(expected, x, Ordering::SeqCst, Ordering::Relaxed);
    res.is_ok() as jboolean
}

/// Class:     sun_misc_Unsafe
/// Method:    compareAndSwapInt
/// Signature: (Ljava/lang/Object{ unimplemented!() }JII)Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapInt(
    env: RawJNIEnv,
    _this: JObject,
    obj: jobject,
    offset: jlong,
    expected: jint,
    x: jint,
) -> jboolean {
    let obj = obj_expect!(env, obj, JNI_FALSE);
    let instance = obj.expect_instance();
    assert_eq!(offset as usize % size_of::<jvalue>(), 0);
    let index = offset as usize / size_of::<jvalue>();

    let mut fields = instance.raw_fields();
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
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _valb: jlong,
    _valc: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getObjectVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)Ljava/lang/Object{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getObjectVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putObjectVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JLjava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putObjectVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _objb: JObject,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getIntVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getIntVolatile(
    env: RawJNIEnv,
    _this: JObject,
    obj: jobject,
    offset: jlong,
) -> jint {
    obj_expect!(env, obj, 0)
        .expect_instance()
        .read_field(offset as usize)
}

/// Class:     sun_misc_Unsafe
/// Method:    putIntVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JI)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putIntVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _offset: jlong,
    _val: jint,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getBooleanVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getBooleanVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putBooleanVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JZ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putBooleanVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _: jboolean,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getByteVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)B
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getByteVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putByteVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JB)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putByteVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _: jbyte,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getShortVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)S
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getShortVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putShortVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JS)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putShortVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _: jshort,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getCharVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)C
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getCharVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putCharVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JC)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putCharVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _: jchar,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getLongVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getLongVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putLongVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putLongVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _offset: jlong,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getFloatVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)F
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getFloatVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putFloatVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JF)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putFloatVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _: jfloat,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    getDoubleVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)D
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getDoubleVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putDoubleVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JD)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putDoubleVolatile(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _: jdouble,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putOrderedObject
/// Signature: (Ljava/lang/Object{ unimplemented!() }JLjava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putOrderedObject(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
    _val: jlong,
    _objb: JObject,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    putOrderedInt
/// Signature: (Ljava/lang/Object{ unimplemented!() }JI)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putOrderedInt(
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
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
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
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
    _env: JNIEnv,
    _this: JObject,
    _obj: JObject,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    park
/// Signature: (ZJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_park(
    _env: JNIEnv,
    _this: JObject,
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
    _env: JNIEnv,
    _this: JObject,
    _: jdoubleArray,
    _val: jint,
) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    loadFence
/// Signature: ()V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_loadFence(_env: JNIEnv, _this: JObject) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    storeFence
/// Signature: ()V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_storeFence(_env: JNIEnv, _this: JObject) {
    unimplemented!()
}

/// Class:     sun_misc_Unsafe
/// Method:    fullFence
/// Signature: ()V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_fullFence(_env: JNIEnv, _this: JObject) {
    unimplemented!()
}
