#![allow(unused_variables)]

use std::ffi::{c_void, CStr, CString, VaList};
use std::mem::{forget, transmute};
use std::ptr::{copy_nonoverlapping, null, null_mut, write_bytes};

use jni::sys::{
    jarray, jboolean, jbyte, jchar, jclass, jdouble, jfieldID, jfloat, jint, jlong, jmethodID,
    jobject, jobjectArray, jobjectRefType, jshort, jsize, jstring, jthrowable, jvalue, jweak,
    va_list, JNIEnv, JNINativeInterface_, JNINativeMethod, JavaVM, JNI_ABORT, JNI_COMMIT, JNI_TRUE,
};

use crate::class::constant::ClassElement;
use crate::class::BufferedRead;
use crate::jvm::call::{FlowControl, JavaEnvInvoke, RawJNIEnv};
use crate::jvm::mem::{
    ArrayReference, FieldDescriptor, JavaPrimitive, JavaTypeEnum, JavaValue,
    ManualInstanceReference, ObjectReference, ObjectType,
};
use crate::jvm::{JavaEnv, ObjectHandle};
use parking_lot::RwLock;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_char;
use std::sync::Arc;
// #[deprecated(note="Switch to storing JVM ptr in JNIEnv::reserved0")]
// pub static mut GLOBAL_JVM: Option<Box<JavaEnv>> = None;

// TODO: Rewrite this function with CStr
pub unsafe extern "system" fn register_natives(
    env: *mut JNIEnv,
    clazz: jclass,
    methods: *const JNINativeMethod,
    num_methods: jint,
) -> jint {
    let env = RawJNIEnv::new(env);

    // debug!("Calling JNIEnv::RegisterNatives");
    let a = ObjectHandle::from_ptr(clazz).unwrap().expect_instance();
    let name_obj: Option<ObjectHandle> = a.lock().read_named_field("name");
    let class = name_obj.unwrap().expect_string().replace('.', "/");
    // let class_object = ObjectHandle::from_ptr(clazz).unwrap();
    // let class = class_object.expect_string();

    let mut registered = 0;

    for method in
        std::slice::from_raw_parts_mut(methods as *mut JNINativeMethod, num_methods as usize)
    {
        let name = CStr::from_ptr(method.name);
        let desc = CStr::from_ptr(method.signature);

        // let jvm = (&**_env).reserved0 as *mut JavaEnv;
        if env.write().linked_libraries.register_fn(
            &class,
            name.to_str().unwrap(),
            desc.to_str().unwrap(),
            method.fnPtr,
        ) {
            registered += 1;
        }
    }

    // forget(class);
    registered
}

unsafe extern "system" fn get_object_class(env: *mut JNIEnv, obj: jobject) -> jclass {
    let env = RawJNIEnv::new(env);

    // let jvm = (**env).reserved0 as *mut JavaEnv;
    match ObjectHandle::from_ptr(obj) {
        Some(v) => env.write().class_instance(&v.get_class()).ptr(),
        None => null_mut(),
    }
}

unsafe extern "system" fn get_method_id(
    env: *mut JNIEnv,
    clazz: jclass,
    name: *const c_char,
    sig: *const c_char,
) -> jmethodID {
    let a = ObjectHandle::from_ptr(clazz).unwrap().expect_instance();
    let name_obj: Option<ObjectHandle> = a.lock().read_named_field("name");

    let name_string = CString::from_raw(name as *mut _);
    let desc_string = CString::from_raw(sig as *mut _);

    let element = ClassElement {
        class: name_obj.unwrap().expect_string().replace('.', "/"),
        element: name_string.to_str().unwrap().to_string(),
        desc: desc_string.to_str().unwrap().to_string(),
    };

    forget(name_string);
    forget(desc_string);

    Box::leak(Box::new(element)) as *mut ClassElement as *mut _
}

unsafe extern "system" fn exception_check(env: *mut JNIEnv) -> jboolean {
    RawJNIEnv::new(env).read_thrown().is_some() as jboolean
    // !(**env).reserved1.is_null() as jboolean
}

unsafe extern "system" fn exception_clear(env: *mut JNIEnv) {
    RawJNIEnv::new(env).write_thrown(None)
    // (&mut **(env as *mut *mut JNINativeInterface_)).reserved1 = null_mut();
}

unsafe extern "system" fn exception_occurred(env: *mut JNIEnv) -> jthrowable {
    RawJNIEnv::new(env).read_thrown().pack().l
    // (&mut **(env as *mut *mut JNINativeInterface_)).reserved1 as _
}

unsafe extern "system" fn exception_describe(env: *mut JNIEnv) {
    eprintln!("*descriptive explanation of exception*")
}

unsafe fn read_method_id(method: jmethodID) -> &'static ClassElement {
    &*(method as *mut ClassElement)
}

unsafe fn read_args(signature: &str, values: *const jvalue) -> Vec<JavaValue> {
    if let Ok(FieldDescriptor::Method { args, .. }) = FieldDescriptor::read_str(signature) {
        let mut ret = Vec::new();

        for (idx, arg) in args.into_iter().enumerate() {
            ret.push(arg.cast(*values.add(idx)).unwrap());
        }

        ret
    } else {
        panic!("Malformed signature!")
    }
}

unsafe extern "system" fn call_obj_method_a(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethodID,
    args: *const jvalue,
) -> jobject {
    let mut env = RawJNIEnv::new(env);
    let target = ObjectHandle::from_ptr(obj).unwrap();
    let element = read_method_id(method_id);
    let parsed_args = read_args(&element.desc, args);

    // let mut jvm = &mut *((**env).reserved0 as *mut JavaEnv);
    match env.invoke_virtual(element.clone(), target, parsed_args) {
        Ok(Some(JavaValue::Reference(v))) => v.pack().l,
        Err(FlowControl::Throws(x)) => {
            env.write_thrown(x);
            // (&mut **(env as *mut *mut JNINativeInterface_)).reserved1 = x.pack().l as _;
            null_mut()
        }
        x => panic!("{:?}", x),
    }
}
// unsafe extern "C" fn call_method(env: *mut JNIEnv,
// obj: jobject,
// method_id: jmethodID,
// ...)
// -> jobject {
//
// }

#[inline]
pub fn build_interface(jvm: &Arc<RwLock<JavaEnv>>) -> JNINativeInterface_ {
    let boxed = Box::new(jvm.clone());
    JNINativeInterface_ {
        // Make a memory leak because its easier than doing things right
        reserved0: Box::leak(boxed) as *mut Arc<RwLock<JavaEnv>> as *mut c_void,
        reserved1: null_mut(),
        reserved2: null_mut(),
        reserved3: null_mut(),
        GetVersion: Some(GetVersion),
        DefineClass: Some(DefineClass),
        FindClass: Some(FindClass),
        FromReflectedMethod: Some(FromReflectedMethod),
        FromReflectedField: Some(FromReflectedField),
        ToReflectedMethod: Some(ToReflectedMethod),
        GetSuperclass: Some(GetSuperclass),
        IsAssignableFrom: Some(IsAssignableFrom),
        ToReflectedField: Some(ToReflectedField),
        Throw: Some(Throw),
        ThrowNew: Some(ThrowNew),
        ExceptionOccurred: Some(exception_occurred),
        ExceptionDescribe: Some(exception_describe),
        ExceptionClear: Some(exception_clear),
        FatalError: Some(FatalError),
        PushLocalFrame: Some(PushLocalFrame),
        PopLocalFrame: Some(PopLocalFrame),
        NewGlobalRef: Some(NewGlobalRef),
        DeleteGlobalRef: Some(DeleteGlobalRef),
        DeleteLocalRef: Some(DeleteLocalRef),
        IsSameObject: Some(IsSameObject),
        NewLocalRef: Some(NewLocalRef),
        EnsureLocalCapacity: Some(EnsureLocalCapacity),
        AllocObject: Some(AllocObject),
        NewObject: None,
        NewObjectV: Some(NewObjectV),
        NewObjectA: Some(NewObjectA),
        GetObjectClass: Some(get_object_class),
        IsInstanceOf: Some(IsInstanceOf),
        GetMethodID: Some(get_method_id),
        CallObjectMethod: Some(CallObjectMethod),
        CallObjectMethodV: Some(CallObjectMethodV),
        CallObjectMethodA: Some(call_obj_method_a),
        CallBooleanMethod: Some(CallBooleanMethod),
        CallBooleanMethodV: Some(CallBooleanMethodV),
        CallBooleanMethodA: Some(CallBooleanMethodA),
        CallByteMethod: Some(CallByteMethod),
        CallByteMethodV: Some(CallByteMethodV),
        CallByteMethodA: Some(CallByteMethodA),
        CallCharMethod: Some(CallCharMethod),
        CallCharMethodV: Some(CallCharMethodV),
        CallCharMethodA: Some(CallCharMethodA),
        CallShortMethod: Some(CallShortMethod),
        CallShortMethodV: Some(CallShortMethodV),
        CallShortMethodA: Some(CallShortMethodA),
        CallIntMethod: Some(CallIntMethod),
        CallIntMethodV: Some(CallIntMethodV),
        CallIntMethodA: Some(CallIntMethodA),
        CallLongMethod: Some(CallLongMethod),
        CallLongMethodV: Some(CallLongMethodV),
        CallLongMethodA: Some(CallLongMethodA),
        CallFloatMethod: Some(CallFloatMethod),
        CallFloatMethodV: Some(CallFloatMethodV),
        CallFloatMethodA: Some(CallFloatMethodA),
        CallDoubleMethod: Some(CallDoubleMethod),
        CallDoubleMethodV: Some(CallDoubleMethodV),
        CallDoubleMethodA: Some(CallDoubleMethodA),
        CallVoidMethod: Some(CallVoidMethod),
        CallVoidMethodV: Some(CallVoidMethodV),
        CallVoidMethodA: Some(CallVoidMethodA),
        CallNonvirtualObjectMethod: Some(CallNonvirtualObjectMethod),
        CallNonvirtualObjectMethodV: Some(CallNonvirtualObjectMethodV),
        CallNonvirtualObjectMethodA: Some(CallNonvirtualObjectMethodA),
        CallNonvirtualBooleanMethod: Some(CallNonvirtualBooleanMethod),
        CallNonvirtualBooleanMethodV: Some(CallNonvirtualBooleanMethodV),
        CallNonvirtualBooleanMethodA: Some(CallNonvirtualBooleanMethodA),
        CallNonvirtualByteMethod: Some(CallNonvirtualByteMethod),
        CallNonvirtualByteMethodV: Some(CallNonvirtualByteMethodV),
        CallNonvirtualByteMethodA: Some(CallNonvirtualByteMethodA),
        CallNonvirtualCharMethod: Some(CallNonvirtualCharMethod),
        CallNonvirtualCharMethodV: Some(CallNonvirtualCharMethodV),
        CallNonvirtualCharMethodA: Some(CallNonvirtualCharMethodA),
        CallNonvirtualShortMethod: Some(CallNonvirtualShortMethod),
        CallNonvirtualShortMethodV: Some(CallNonvirtualShortMethodV),
        CallNonvirtualShortMethodA: Some(CallNonvirtualShortMethodA),
        CallNonvirtualIntMethod: Some(CallNonvirtualIntMethod),
        CallNonvirtualIntMethodV: Some(CallNonvirtualIntMethodV),
        CallNonvirtualIntMethodA: Some(CallNonvirtualIntMethodA),
        CallNonvirtualLongMethod: Some(CallNonvirtualLongMethod),
        CallNonvirtualLongMethodV: Some(CallNonvirtualLongMethodV),
        CallNonvirtualLongMethodA: Some(CallNonvirtualLongMethodA),
        CallNonvirtualFloatMethod: Some(CallNonvirtualFloatMethod),
        CallNonvirtualFloatMethodV: Some(CallNonvirtualFloatMethodV),
        CallNonvirtualFloatMethodA: Some(CallNonvirtualFloatMethodA),
        CallNonvirtualDoubleMethod: Some(CallNonvirtualDoubleMethod),
        CallNonvirtualDoubleMethodV: Some(CallNonvirtualDoubleMethodV),
        CallNonvirtualDoubleMethodA: Some(CallNonvirtualDoubleMethodA),
        CallNonvirtualVoidMethod: Some(CallNonvirtualVoidMethod),
        CallNonvirtualVoidMethodV: Some(CallNonvirtualVoidMethodV),
        CallNonvirtualVoidMethodA: Some(CallNonvirtualVoidMethodA),
        GetFieldID: Some(GetFieldID),
        GetObjectField: Some(GetObjectField),
        GetBooleanField: Some(GetBooleanField),
        GetByteField: Some(GetByteField),
        GetCharField: Some(GetCharField),
        GetShortField: Some(GetShortField),
        GetIntField: Some(GetIntField),
        GetLongField: Some(GetLongField),
        GetFloatField: Some(GetFloatField),
        GetDoubleField: Some(GetDoubleField),
        SetObjectField: Some(SetObjectField),
        SetBooleanField: Some(SetBooleanField),
        SetByteField: Some(SetByteField),
        SetCharField: Some(SetCharField),
        SetShortField: Some(SetShortField),
        SetIntField: Some(SetIntField),
        SetLongField: Some(SetLongField),
        SetFloatField: Some(SetFloatField),
        SetDoubleField: Some(SetDoubleField),
        GetStaticMethodID: Some(GetStaticMethodID),
        CallStaticObjectMethod: Some(CallStaticObjectMethod),
        CallStaticObjectMethodV: Some(CallStaticObjectMethodV),
        CallStaticObjectMethodA: Some(CallStaticObjectMethodA),
        CallStaticBooleanMethod: Some(CallStaticBooleanMethod),
        CallStaticBooleanMethodV: Some(CallStaticBooleanMethodV),
        CallStaticBooleanMethodA: Some(CallStaticBooleanMethodA),
        CallStaticByteMethod: Some(CallStaticByteMethod),
        CallStaticByteMethodV: Some(CallStaticByteMethodV),
        CallStaticByteMethodA: Some(CallStaticByteMethodA),
        CallStaticCharMethod: Some(CallStaticCharMethod),
        CallStaticCharMethodV: Some(CallStaticCharMethodV),
        CallStaticCharMethodA: Some(CallStaticCharMethodA),
        CallStaticShortMethod: Some(CallStaticShortMethod),
        CallStaticShortMethodV: Some(CallStaticShortMethodV),
        CallStaticShortMethodA: Some(CallStaticShortMethodA),
        CallStaticIntMethod: Some(CallStaticIntMethod),
        CallStaticIntMethodV: Some(CallStaticIntMethodV),
        CallStaticIntMethodA: Some(CallStaticIntMethodA),
        CallStaticLongMethod: Some(CallStaticLongMethod),
        CallStaticLongMethodV: Some(CallStaticLongMethodV),
        CallStaticLongMethodA: Some(CallStaticLongMethodA),
        CallStaticFloatMethod: Some(CallStaticFloatMethod),
        CallStaticFloatMethodV: Some(CallStaticFloatMethodV),
        CallStaticFloatMethodA: Some(CallStaticFloatMethodA),
        CallStaticDoubleMethod: Some(CallStaticDoubleMethod),
        CallStaticDoubleMethodV: Some(CallStaticDoubleMethodV),
        CallStaticDoubleMethodA: Some(CallStaticDoubleMethodA),
        CallStaticVoidMethod: Some(CallStaticVoidMethod),
        CallStaticVoidMethodV: Some(CallStaticVoidMethodV),
        CallStaticVoidMethodA: Some(CallStaticVoidMethodA),
        GetStaticFieldID: Some(GetStaticFieldID),
        GetStaticObjectField: Some(GetStaticObjectField),
        GetStaticBooleanField: Some(GetStaticBooleanField),
        GetStaticByteField: Some(GetStaticByteField),
        GetStaticCharField: Some(GetStaticCharField),
        GetStaticShortField: Some(GetStaticShortField),
        GetStaticIntField: Some(GetStaticIntField),
        GetStaticLongField: Some(GetStaticLongField),
        GetStaticFloatField: Some(GetStaticFloatField),
        GetStaticDoubleField: Some(GetStaticDoubleField),
        SetStaticObjectField: Some(SetStaticObjectField),
        SetStaticBooleanField: Some(SetStaticBooleanField),
        SetStaticByteField: Some(SetStaticByteField),
        SetStaticCharField: Some(SetStaticCharField),
        SetStaticShortField: Some(SetStaticShortField),
        SetStaticIntField: Some(SetStaticIntField),
        SetStaticLongField: Some(SetStaticLongField),
        SetStaticFloatField: Some(SetStaticFloatField),
        SetStaticDoubleField: Some(SetStaticDoubleField),
        NewString: Some(NewString),
        GetStringLength: Some(GetStringLength),
        GetStringChars: Some(GetStringChars),
        ReleaseStringChars: Some(ReleaseStringChars),
        NewStringUTF: Some(NewStringUTF),
        GetStringUTFLength: Some(GetStringUTFLength),
        GetStringUTFChars: Some(GetStringUTFChars),
        ReleaseStringUTFChars: Some(ReleaseStringUTFChars),
        GetArrayLength: Some(GetArrayLength),
        NewObjectArray: Some(NewObjectArray),
        GetObjectArrayElement: Some(GetObjectArrayElement),
        SetObjectArrayElement: Some(SetObjectArrayElement),
        NewBooleanArray: Some(NewBooleanArray),
        NewByteArray: Some(NewByteArray),
        NewCharArray: Some(NewCharArray),
        NewShortArray: Some(NewShortArray),
        NewIntArray: Some(NewIntArray),
        NewLongArray: Some(NewLongArray),
        NewFloatArray: Some(NewFloatArray),
        NewDoubleArray: Some(NewDoubleArray),
        GetBooleanArrayElements: Some(GetBooleanArrayElements),
        GetByteArrayElements: Some(GetByteArrayElements),
        GetCharArrayElements: Some(GetCharArrayElements),
        GetShortArrayElements: Some(GetShortArrayElements),
        GetIntArrayElements: Some(GetIntArrayElements),
        GetLongArrayElements: Some(GetLongArrayElements),
        GetFloatArrayElements: Some(GetFloatArrayElements),
        GetDoubleArrayElements: Some(GetDoubleArrayElements),
        ReleaseBooleanArrayElements: Some(ReleaseBooleanArrayElements),
        ReleaseByteArrayElements: Some(ReleaseByteArrayElements),
        ReleaseCharArrayElements: Some(ReleaseCharArrayElements),
        ReleaseShortArrayElements: Some(ReleaseShortArrayElements),
        ReleaseIntArrayElements: Some(ReleaseIntArrayElements),
        ReleaseLongArrayElements: Some(ReleaseLongArrayElements),
        ReleaseFloatArrayElements: Some(ReleaseFloatArrayElements),
        ReleaseDoubleArrayElements: Some(ReleaseDoubleArrayElements),
        GetBooleanArrayRegion: Some(GetBooleanArrayRegion),
        GetByteArrayRegion: Some(GetByteArrayRegion),
        GetCharArrayRegion: Some(GetCharArrayRegion),
        GetShortArrayRegion: Some(GetShortArrayRegion),
        GetIntArrayRegion: Some(GetIntArrayRegion),
        GetLongArrayRegion: Some(GetLongArrayRegion),
        GetFloatArrayRegion: Some(GetFloatArrayRegion),
        GetDoubleArrayRegion: Some(GetDoubleArrayRegion),
        SetBooleanArrayRegion: Some(SetBooleanArrayRegion),
        SetByteArrayRegion: Some(SetByteArrayRegion),
        SetCharArrayRegion: Some(SetCharArrayRegion),
        SetShortArrayRegion: Some(SetShortArrayRegion),
        SetIntArrayRegion: Some(SetIntArrayRegion),
        SetLongArrayRegion: Some(SetLongArrayRegion),
        SetFloatArrayRegion: Some(SetFloatArrayRegion),
        SetDoubleArrayRegion: Some(SetDoubleArrayRegion),
        RegisterNatives: Some(register_natives),
        UnregisterNatives: Some(UnregisterNatives),
        MonitorEnter: Some(MonitorEnter),
        MonitorExit: Some(MonitorExit),
        GetJavaVM: Some(GetJavaVM),
        GetStringRegion: Some(GetStringRegion),
        GetStringUTFRegion: Some(GetStringUTFRegion),
        GetPrimitiveArrayCritical: Some(GetPrimitiveArrayCritical),
        ReleasePrimitiveArrayCritical: Some(ReleasePrimitiveArrayCritical),
        GetStringCritical: Some(GetStringCritical),
        ReleaseStringCritical: Some(ReleaseStringCritical),
        NewWeakGlobalRef: Some(NewWeakGlobalRef),
        DeleteWeakGlobalRef: Some(DeleteWeakGlobalRef),
        ExceptionCheck: Some(exception_check),
        NewDirectByteBuffer: Some(NewDirectByteBuffer),
        GetDirectBufferAddress: Some(GetDirectBufferAddress),
        GetDirectBufferCapacity: Some(GetDirectBufferCapacity),
        GetObjectRefType: Some(GetObjectRefType),
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetVersion(env: *mut JNIEnv) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn DefineClass(
    env: *mut JNIEnv,
    name: *const c_char,
    loader: jobject,
    buf: *const jbyte,
    len: jsize,
) -> jclass {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn FindClass(env: *mut JNIEnv, name: *const c_char) -> jclass {
    let env = RawJNIEnv::new(env);
    let name_cstr = CStr::from_ptr(name);
    let mut lock = env.write();
    lock.class_loader
        .attempt_load(&name_cstr.to_string_lossy())
        .unwrap();
    lock.class_instance(&name_cstr.to_string_lossy()).ptr()
}

#[no_mangle]
pub unsafe extern "system" fn FromReflectedMethod(env: *mut JNIEnv, method: jobject) -> jmethodID {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn FromReflectedField(env: *mut JNIEnv, field: jobject) -> jfieldID {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn ToReflectedMethod(
    env: *mut JNIEnv,
    cls: jclass,
    method_id: jmethodID,
    is_static: jboolean,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn GetSuperclass(env: *mut JNIEnv, sub: jclass) -> jclass {
    let env = RawJNIEnv::new(env);
    let class = obj_expect!(env, sub, null_mut()).unwrap_as_class();

    if class == "java/lang/Object" {
        // Idk how this case is supposed to be handled
        return null_mut();
    }

    let mut lock = env.write();
    let raw_class = lock.class_loader.class(&class).unwrap();
    let super_class = raw_class.super_class();
    lock.class_instance(&super_class).ptr()
}

#[no_mangle]
pub unsafe extern "system" fn IsAssignableFrom(
    env: *mut JNIEnv,
    sub: jclass,
    sup: jclass,
) -> jboolean {
    let env = RawJNIEnv::new(env);
    let subclass = obj_expect!(env, sub).unwrap_as_class();
    let supclass = obj_expect!(env, sup).unwrap_as_class();

    // TODO: Idk if this may cause issues for me later
    let jvm = env.read();
    matches!(jvm.instanceof(&subclass, &supclass), Some(true)) as jboolean
}

#[no_mangle]
pub unsafe extern "system" fn ToReflectedField(
    env: *mut JNIEnv,
    cls: jclass,
    field_id: jfieldID,
    is_static: jboolean,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn Throw(env: *mut JNIEnv, obj: jthrowable) -> jint {
    let env = RawJNIEnv::new(env);
    env.write_thrown(std::mem::transmute(obj));
    1 // what do we return?
}

#[no_mangle]
pub unsafe extern "system" fn ThrowNew(
    env: *mut JNIEnv,
    clazz: jclass,
    msg: *const c_char,
) -> jint {
    unimplemented!("Attempting to throw new {:?}", CStr::from_ptr(msg))
}

#[no_mangle]
pub unsafe extern "system" fn ExceptionOccurred(env: *mut JNIEnv) -> jthrowable {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn ExceptionDescribe(env: *mut JNIEnv) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn ExceptionClear(env: *mut JNIEnv) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn FatalError(env: *mut JNIEnv, msg: *const c_char) -> ! {
    panic!("Fatal Error: {}", CStr::from_ptr(msg).to_string_lossy())
}

#[no_mangle]
pub unsafe extern "system" fn PushLocalFrame(env: *mut JNIEnv, capacity: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn PopLocalFrame(env: *mut JNIEnv, result: jobject) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn NewGlobalRef(env: *mut JNIEnv, lobj: jobject) -> jobject {
    // TODO: Once GC is working, this should ensure a global reference exists for a given object
    lobj
}

#[no_mangle]
pub unsafe extern "system" fn DeleteGlobalRef(env: *mut JNIEnv, gref: jobject) {
    // TODO: Once GC is working, this should remove a global reference from NewGlobalRef
}

#[no_mangle]
pub unsafe extern "system" fn DeleteLocalRef(env: *mut JNIEnv, obj: jobject) {}

#[no_mangle]
pub unsafe extern "system" fn IsSameObject(
    env: *mut JNIEnv,
    obj1: jobject,
    obj2: jobject,
) -> jboolean {
    (ObjectHandle::from_ptr(obj1) == ObjectHandle::from_ptr(obj2)) as jboolean
}

#[no_mangle]
pub unsafe extern "system" fn NewLocalRef(env: *mut JNIEnv, ref_: jobject) -> jobject {
    unimplemented!()
}

/// Due to hotspot jvm limitations it can only support a limited number of local references on a
/// thread at a any time. This JVM does not have that limitation so always return success.
#[no_mangle]
pub unsafe extern "system" fn EnsureLocalCapacity(env: *mut JNIEnv, capacity: jint) -> jint {
    0
}

#[no_mangle]
pub unsafe extern "system" fn AllocObject(env: *mut JNIEnv, clazz: jclass) -> jobject {
    let env = RawJNIEnv::new(env);
    let class_name = obj_expect!(env, clazz, null_mut()).unwrap_as_class();
    let schema = env.write().class_schema(&class_name);
    ObjectHandle::new(schema).ptr()
}

#[no_mangle]
pub unsafe extern "system" fn NewObjectV(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethodID,
    args: va_list,
) -> jobject {
    let obj = (**env).AllocObject.unwrap()(env, clazz);
    (**env).CallVoidMethodV.unwrap()(env, obj, method_id, args);
    obj
}

#[no_mangle]
pub unsafe extern "system" fn NewObjectA(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethodID,
    args: *const jvalue,
) -> jobject {
    let obj = (**env).AllocObject.unwrap()(env, clazz);
    (**env).CallVoidMethodA.unwrap()(env, obj, method_id, args);
    obj
}

#[no_mangle]
pub unsafe extern "system" fn GetObjectClass(env: *mut JNIEnv, obj: jobject) -> jclass {
    let env = RawJNIEnv::new(env);
    let class_name = obj_expect!(env, obj, null_mut()).get_class();
    let mut jvm = env.write();
    jvm.class_instance(&class_name).ptr()
}

#[no_mangle]
pub unsafe extern "system" fn IsInstanceOf(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
) -> jboolean {
    let env = RawJNIEnv::new(env);
    let obj_class = obj_expect!(env, obj).get_class();
    let super_class = obj_expect!(env, clazz).unwrap_as_class();
    let jvm = env.read();
    matches!(jvm.instanceof(&obj_class, &super_class), Some(true)) as jboolean
}

#[no_mangle]
pub unsafe extern "system" fn GetFieldID(
    env: *mut JNIEnv,
    clazz: jclass,
    name: *const c_char,
    sig: *const c_char,
) -> jfieldID {
    let env = RawJNIEnv::new(env);
    let name = CStr::from_ptr(name);
    let sig = CStr::from_ptr(sig);

    let class = obj_expect!(env, clazz, null_mut());
    let target_class = class.unwrap_as_class();

    let raw_class = env
        .read()
        .class_loader
        .class(&target_class)
        .unwrap()
        .to_owned();
    let raw_field = raw_class
        .get_field(&name.to_string_lossy(), &sig.to_string_lossy())
        .unwrap();
    // .unwrap_as_class();

    let target_class_schema = env.write().class_schema(&target_class);
    let field_schema = env.write().class_schema("java/lang/reflect/Field");
    let field_obj = ObjectHandle::new(field_schema.clone());
    let instance = field_obj.expect_instance();
    let mut lock = instance.lock();
    lock.write_named_field("clazz", Some(class));

    // let name = raw_field.name(&raw_class.constants).unwrap();
    let slot = match target_class_schema
        .field_offsets
        .get(&*name.to_string_lossy())
    {
        Some(v) => v.offset as jint,
        None => -1,
    };
    lock.write_named_field("slot", slot);
    let mut jvm = env.write();
    lock.write_named_field("name", jvm.build_string(&name.to_string_lossy()));

    let type_class = match FieldDescriptor::read_str(&sig.to_string_lossy()).unwrap() {
        FieldDescriptor::Byte => jvm.class_instance("byte"),
        FieldDescriptor::Char => jvm.class_instance("char"),
        FieldDescriptor::Double => jvm.class_instance("double"),
        FieldDescriptor::Float => jvm.class_instance("float"),
        FieldDescriptor::Int => jvm.class_instance("int"),
        FieldDescriptor::Long => jvm.class_instance("long"),
        FieldDescriptor::Short => jvm.class_instance("short"),
        FieldDescriptor::Boolean => jvm.class_instance("boolean"),
        FieldDescriptor::Object(x) => jvm.class_instance(&x),
        FieldDescriptor::Array(x) => jvm.class_instance(&format!("[{:?}", x)),
        _ => panic!("Can't get classes for these types"),
    };
    lock.write_named_field("type", Some(type_class));
    lock.write_named_field("modifiers", raw_field.access.bits() as jint);
    field_obj.ptr() as jfieldID
}

#[no_mangle]
pub unsafe extern "system" fn GetObjectField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfieldID,
) -> jobject {
    let env = RawJNIEnv::new(env);
    let obj = obj_expect!(env, obj, null_mut()).expect_instance();
    let field = obj_expect!(env, field_id as jobject, null_mut()).expect_instance();
    let field_name: Option<ObjectHandle> = field.lock().read_named_field("name");
    let out: Option<ObjectHandle> = obj
        .lock()
        .read_named_field(field_name.unwrap().expect_string());
    out.pack().l
}

#[no_mangle]
pub unsafe extern "system" fn SetObjectField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfieldID,
    val: jobject,
) {
    let env = RawJNIEnv::new(env);
    let obj = obj_expect!(env, obj).expect_instance();
    let field = obj_expect!(env, field_id as jobject).expect_instance();
    let field_name: Option<ObjectHandle> = field.lock().read_named_field("name");
    obj.lock().write_named_field(
        field_name.unwrap().expect_string(),
        ObjectHandle::from_ptr(val),
    );
}

macro_rules! impl_obj_field {
    ($type:ty: $get:ident, $set:ident) => {
        #[no_mangle]
        pub unsafe extern "system" fn $get(
            env: *mut JNIEnv,
            obj: jobject,
            field_id: jfieldID,
        ) -> $type {
            let env = RawJNIEnv::new(env);
            let obj = obj_expect!(env, obj).expect_instance();
            let obj_lock = obj.lock();
            let field = obj_expect!(env, field_id as jobject).expect_instance();
            let field_name: Option<ObjectHandle> = field.lock().read_named_field("name");
            obj_lock.read_named_field(field_name.unwrap().expect_string())
        }

        #[no_mangle]
        pub unsafe extern "system" fn $set(
            env: *mut JNIEnv,
            obj: jobject,
            field_id: jfieldID,
            val: $type,
        ) {
            let env = RawJNIEnv::new(env);
            let obj = obj_expect!(env, obj).expect_instance();
            let mut obj_lock = obj.lock();
            let field = obj_expect!(env, field_id as jobject).expect_instance();
            let field_name: Option<ObjectHandle> = field.lock().read_named_field("name");
            obj_lock.write_named_field(field_name.unwrap().expect_string(), val);
        }
    };
}

impl_obj_field!(jboolean: GetBooleanField, SetBooleanField);
impl_obj_field!(jbyte: GetByteField, SetByteField);
impl_obj_field!(jchar: GetCharField, SetCharField);
impl_obj_field!(jshort: GetShortField, SetShortField);
impl_obj_field!(jint: GetIntField, SetIntField);
impl_obj_field!(jlong: GetLongField, SetLongField);
impl_obj_field!(jfloat: GetFloatField, SetFloatField);
impl_obj_field!(jdouble: GetDoubleField, SetDoubleField);

#[no_mangle]
pub unsafe extern "system" fn GetStaticMethodID(
    env: *mut JNIEnv,
    clazz: jclass,
    name: *const c_char,
    sig: *const c_char,
) -> jmethodID {
    let a = ObjectHandle::from_ptr(clazz).unwrap().expect_instance();
    let name_obj: Option<ObjectHandle> = a.lock().read_named_field("name");

    let name_string = CStr::from_ptr(name);
    let desc_string = CStr::from_ptr(sig);

    let element = ClassElement {
        class: name_obj.unwrap().expect_string().replace('.', "/"),
        element: name_string.to_str().unwrap().to_string(),
        desc: desc_string.to_str().unwrap().to_string(),
    };

    Box::leak(Box::new(element)) as *mut ClassElement as *mut _
}

macro_rules! impl_call {
    ($type:tt ($fn:ident, $fnv:ident, $fna:ident)($($out_match:tt)+) -> $out:ty) => {
        impl_call!($type ($fn, $fnv, $fna)($($out_match)+) -> $out | Default::default());
    };
    ($type:tt ($fn:ident, $fnv:ident, $fna:ident)($($out_match:tt)+) -> $out:ty | $default:expr) => {
        impl_call!{@impl $type extern "C" $fn(mut args: ...)(|x| {
            let mut out_vals = Vec::with_capacity(x.len());
            for arg in x {
                let va_value: jvalue = transmute(args.arg::<u64>());
                out_vals.push(arg.cast(va_value).unwrap());
            }
            out_vals
        })($($out_match)+) -> $out | $default}
        impl_call!{@impl $type extern "system" $fnv(args: va_list)(|x| {
            let mut va_args: VaList = transmute(args);
            let mut out_vals = Vec::with_capacity(x.len());
            for arg in x {
                let va_value: jvalue = transmute(va_args.arg::<u64>());
                out_vals.push(arg.cast(va_value).unwrap());
            }
            out_vals
        })($($out_match)+) -> $out | $default}
        impl_call!{@impl $type extern "system" $fna(args: *const jvalue)(|x| {
            let mut out_vals = Vec::with_capacity(x.len());
            for (idx, arg) in x.into_iter().enumerate() {
                out_vals.push(arg.cast(*args.add(idx)).unwrap());
            }
            out_vals
        })($($out_match)+) -> $out | $default}
    };
    (@impl static extern $extern:literal $fn:ident($($fn_args:tt)+)(|$args:ident| $load_args:expr)($($out_match:tt)+) -> $out:ty | $default:expr) => {
        #[no_mangle]
        pub unsafe extern $extern fn $fn(
            env: *mut JNIEnv,
            clazz: jclass,
            method_id: jmethodID,
            $($fn_args)+
        ) -> $out {
            let element = (&*(method_id as *mut ClassElement)).clone();

            let java_args = if let Ok(FieldDescriptor::Method {args: $args, returns}) = FieldDescriptor::read_str(&element.desc) {
                $load_args
            } else {
                panic!("Invalid field descriptor for method");
            };

            let mut env = RawJNIEnv::new(env);
            match env.invoke_static(element, java_args) {
                $($out_match)+
                Err(FlowControl::Throws(x)) => {
                    env.write_thrown(x);
                    $default
                }
                x => panic!("{:?}", x),
            }
        }
    };
    (@impl virtual extern $extern:literal $fn:ident($($fn_args:tt)+)(|$args:ident| $load_args:expr)($($out_match:tt)+) -> $out:ty | $default:expr) => {
        #[no_mangle]
        pub unsafe extern $extern fn $fn(
            env: *mut JNIEnv,
            obj: jobject,
            method_id: jmethodID,
            $($fn_args)+
        ) -> $out {
            let element = (&*(method_id as *mut ClassElement)).clone();

            let java_args = if let Ok(FieldDescriptor::Method {args: $args, returns}) = FieldDescriptor::read_str(&element.desc) {
                $load_args
            } else {
                panic!("Invalid field descriptor for method");
            };

            let mut env = RawJNIEnv::new(env);
            let target = obj_expect!(env, obj, $default);
            match env.invoke_virtual(element, target, java_args) {
                $($out_match)+
                Err(FlowControl::Throws(x)) => {
                    env.write_thrown(x);
                    $default
                }
                x => panic!("{:?}", x),
            }
        }
    };
    (@impl special extern $extern:literal $fn:ident($($fn_args:tt)+)(|$args:ident| $load_args:expr)($($out_match:tt)+) -> $out:ty | $default:expr) => {
        #[no_mangle]
        pub unsafe extern $extern fn $fn(
            env: *mut JNIEnv,
            obj: jobject,
            _cls: jclass,
            method_id: jmethodID,
            $($fn_args)+
        ) -> $out {
            let element = (&*(method_id as *mut ClassElement)).clone();

            let java_args = if let Ok(FieldDescriptor::Method {args: $args, returns}) = FieldDescriptor::read_str(&element.desc) {
                $load_args
            } else {
                panic!("Invalid field descriptor for method");
            };

            let mut env = RawJNIEnv::new(env);
            let target = obj_expect!(env, obj, $default);
            match env.invoke_special(element, target, java_args) {
                $($out_match)+
                Err(FlowControl::Throws(x)) => {
                    env.write_thrown(x);
                    $default
                }
                x => panic!("{:?}", x),
            }
        }
    };
}

impl_call!(static (CallStaticObjectMethod, CallStaticObjectMethodV, CallStaticObjectMethodA)(Ok(Some(JavaValue::Reference(x))) => x.pack().l,) -> jobject | null_mut());
impl_call!(static (CallStaticBooleanMethod, CallStaticBooleanMethodV, CallStaticBooleanMethodA)(Ok(Some(JavaValue::Byte(x))) => x as _, Ok(Some(JavaValue::Int(x))) => x as _,) -> jboolean);
impl_call!(static (CallStaticByteMethod, CallStaticByteMethodV, CallStaticByteMethodA)(Ok(Some(JavaValue::Byte(x))) => x,) -> jbyte);
impl_call!(static (CallStaticCharMethod, CallStaticCharMethodV, CallStaticCharMethodA)(Ok(Some(JavaValue::Char(x))) => x,) -> jchar);
impl_call!(static (CallStaticShortMethod, CallStaticShortMethodV, CallStaticShortMethodA)(Ok(Some(JavaValue::Short(x))) => x,) -> jshort);
impl_call!(static (CallStaticIntMethod, CallStaticIntMethodV, CallStaticIntMethodA)(Ok(Some(JavaValue::Int(x))) => x,) -> jint);
impl_call!(static (CallStaticLongMethod, CallStaticLongMethodV, CallStaticLongMethodA)(Ok(Some(JavaValue::Long(x))) => x,) -> jlong);
impl_call!(static (CallStaticFloatMethod, CallStaticFloatMethodV, CallStaticFloatMethodA)(Ok(Some(JavaValue::Float(x))) => x,) -> jfloat);
impl_call!(static (CallStaticDoubleMethod, CallStaticDoubleMethodV, CallStaticDoubleMethodA)(Ok(Some(JavaValue::Double(x))) => x,) -> jdouble);
impl_call!(static (CallStaticVoidMethod, CallStaticVoidMethodV, CallStaticVoidMethodA)(Ok(None) => {},) -> ());

impl_call!(virtual (CallObjectMethod, CallObjectMethodV, CallObjectMethodA)(Ok(Some(JavaValue::Reference(x))) => x.pack().l,) -> jobject | null_mut());
impl_call!(virtual (CallBooleanMethod, CallBooleanMethodV, CallBooleanMethodA)(Ok(Some(JavaValue::Byte(x))) => x as _,) -> jboolean);
impl_call!(virtual (CallByteMethod, CallByteMethodV, CallByteMethodA)(Ok(Some(JavaValue::Byte(x))) => x,) -> jbyte);
impl_call!(virtual (CallCharMethod, CallCharMethodV, CallCharMethodA)(Ok(Some(JavaValue::Char(x))) => x,) -> jchar);
impl_call!(virtual (CallShortMethod, CallShortMethodV, CallShortMethodA)(Ok(Some(JavaValue::Short(x))) => x,) -> jshort);
impl_call!(virtual (CallIntMethod, CallIntMethodV, CallIntMethodA)(Ok(Some(JavaValue::Int(x))) => x,) -> jint);
impl_call!(virtual (CallLongMethod, CallLongMethodV, CallLongMethodA)(Ok(Some(JavaValue::Long(x))) => x,) -> jlong);
impl_call!(virtual (CallFloatMethod, CallFloatMethodV, CallFloatMethodA)(Ok(Some(JavaValue::Float(x))) => x,) -> jfloat);
impl_call!(virtual (CallDoubleMethod, CallDoubleMethodV, CallDoubleMethodA)(Ok(Some(JavaValue::Double(x))) => x,) -> jdouble);
impl_call!(virtual (CallVoidMethod, CallVoidMethodV, CallVoidMethodA)(Ok(None) => {},) -> ());

impl_call!(special (CallNonvirtualObjectMethod, CallNonvirtualObjectMethodV, CallNonvirtualObjectMethodA)(Ok(Some(JavaValue::Reference(x))) => x.pack().l,) -> jobject | null_mut());
impl_call!(special (CallNonvirtualBooleanMethod, CallNonvirtualBooleanMethodV, CallNonvirtualBooleanMethodA)(Ok(Some(JavaValue::Byte(x))) => x as _,) -> jboolean);
impl_call!(special (CallNonvirtualByteMethod, CallNonvirtualByteMethodV, CallNonvirtualByteMethodA)(Ok(Some(JavaValue::Byte(x))) => x,) -> jbyte);
impl_call!(special (CallNonvirtualCharMethod, CallNonvirtualCharMethodV, CallNonvirtualCharMethodA)(Ok(Some(JavaValue::Char(x))) => x,) -> jchar);
impl_call!(special (CallNonvirtualShortMethod, CallNonvirtualShortMethodV, CallNonvirtualShortMethodA)(Ok(Some(JavaValue::Short(x))) => x,) -> jshort);
impl_call!(special (CallNonvirtualIntMethod, CallNonvirtualIntMethodV, CallNonvirtualIntMethodA)(Ok(Some(JavaValue::Int(x))) => x,) -> jint);
impl_call!(special (CallNonvirtualLongMethod, CallNonvirtualLongMethodV, CallNonvirtualLongMethodA)(Ok(Some(JavaValue::Long(x))) => x,) -> jlong);
impl_call!(special (CallNonvirtualFloatMethod, CallNonvirtualFloatMethodV, CallNonvirtualFloatMethodA)(Ok(Some(JavaValue::Float(x))) => x,) -> jfloat);
impl_call!(special (CallNonvirtualDoubleMethod, CallNonvirtualDoubleMethodV, CallNonvirtualDoubleMethodA)(Ok(Some(JavaValue::Double(x))) => x,) -> jdouble);
impl_call!(special (CallNonvirtualVoidMethod, CallNonvirtualVoidMethodV, CallNonvirtualVoidMethodA)(Ok(None) => {},) -> ());

#[no_mangle]
pub unsafe extern "system" fn GetStaticFieldID(
    env: *mut JNIEnv,
    clazz: jclass,
    name: *const c_char,
    sig: *const c_char,
) -> jfieldID {
    (**env).GetFieldID.unwrap()(env, clazz, name, sig)
}

macro_rules! impl_static_field {
    ($type:ty, $get:ident($($match_tokens:tt)+), $set:ident(|$x:ident| $y:expr)) => {
        impl_static_field!{$type, $get($($match_tokens)+), $set(|$x| $y), Default::default()}
    };
    ($type:ty, $get:ident($($match_tokens:tt)+), $set:ident(|$x:ident| $y:expr), $default:expr) => {
        #[no_mangle]
        pub unsafe extern "system" fn $get(
            env: *mut JNIEnv,
            clazz: jclass,
            field_id: jfieldID,
        ) -> $type {
            let env = RawJNIEnv::new(env);
            let field = obj_expect!(env, field_id as jobject, $default).expect_instance();
            let field_name: Option<ObjectHandle> = field.lock().read_named_field("name");
            let class = obj_expect!(env, clazz, $default).unwrap_as_class();

            let jvm = env.read();
            match jvm.static_fields.get_static(&field_name.unwrap().expect_string(), &class) {
                $($match_tokens)+,
                x => panic!("Static not found: {:?}", x),
            }
        }


        #[no_mangle]
        pub unsafe extern "system" fn $set(
            env: *mut JNIEnv,
            clazz: jclass,
            field_id: jfieldID,
            $x: $type,
        ) {
            let env = RawJNIEnv::new(env);
            let field = obj_expect!(env, field_id as jobject).expect_instance();
            let field_name: Option<ObjectHandle> = field.lock().read_named_field("name");
            let class = obj_expect!(env, clazz).unwrap_as_class();

            env.write().static_fields.set_static(&field_name.unwrap().expect_string(), &class, $y);
        }
    };
}

impl_static_field!(jobject, GetStaticObjectField(Some(JavaValue::Reference(x)) => x.pack().l), SetStaticObjectField(|x| JavaValue::Reference(ObjectHandle::from_ptr(x))), null_mut());
impl_static_field!(jboolean, GetStaticBooleanField(Some(JavaValue::Byte(x)) => x as _), SetStaticBooleanField(|x| JavaValue::Byte(x as _)));
impl_static_field!(jbyte, GetStaticByteField(Some(JavaValue::Byte(x)) => x), SetStaticByteField(|x| JavaValue::Byte(x)));
impl_static_field!(jchar, GetStaticCharField(Some(JavaValue::Char(x)) => x), SetStaticCharField(|x| JavaValue::Char(x)));
impl_static_field!(jshort, GetStaticShortField(Some(JavaValue::Short(x)) => x), SetStaticShortField(|x| JavaValue::Short(x)));
impl_static_field!(jint, GetStaticIntField(Some(JavaValue::Int(x)) => x), SetStaticIntField(|x| JavaValue::Int(x)));
impl_static_field!(jlong, GetStaticLongField(Some(JavaValue::Long(x)) => x), SetStaticLongField(|x| JavaValue::Long(x)));
impl_static_field!(jfloat, GetStaticFloatField(Some(JavaValue::Float(x)) => x), SetStaticFloatField(|x| JavaValue::Float(x)));
impl_static_field!(jdouble, GetStaticDoubleField(Some(JavaValue::Double(x)) => x), SetStaticDoubleField(|x| JavaValue::Double(x)));

#[no_mangle]
pub unsafe extern "system" fn NewString(
    env: *mut JNIEnv,
    unicode: *const jchar,
    len: jsize,
) -> jstring {
    let env = RawJNIEnv::new(env);
    let handle = ObjectHandle::new(env.write().class_schema("java/lang/String"));
    let object = handle.expect_instance();

    let mut chars = vec![0; len as usize];
    unicode.copy_to(chars.as_mut_ptr(), len as usize);

    object
        .lock()
        .write_named_field("value", Some(ObjectHandle::array_from_data(chars)));
    handle.ptr()
}

#[no_mangle]
pub unsafe extern "system" fn GetStringLength(env: *mut JNIEnv, str: jstring) -> jsize {
    let env = RawJNIEnv::new(env);
    obj_expect!(env, str, 0).expect_string().len() as jsize
}

#[no_mangle]
pub unsafe extern "system" fn GetStringChars(
    env: *mut JNIEnv,
    str: jstring,
    is_copy: *mut jboolean,
) -> *const jchar {
    let env = RawJNIEnv::new(env);
    if !is_copy.is_null() {
        *is_copy = JNI_TRUE;
    }

    let arr: Option<ObjectHandle> = obj_expect!(env, str, null_mut())
        .expect_instance()
        .lock()
        .read_named_field("value");
    let mut clone = arr.unwrap().expect_array::<jchar>().lock().to_vec();
    let ret = clone.as_mut_ptr();
    forget(clone); // Forget it so it can be recovered later
    ret
}

#[no_mangle]
pub unsafe extern "system" fn ReleaseStringChars(
    env: *mut JNIEnv,
    str: jstring,
    chars: *const jchar,
) {
    let env = RawJNIEnv::new(env);
    let obj: Option<ObjectHandle> = obj_expect!(env, str)
        .expect_instance()
        .lock()
        .read_named_field("value");
    let arr = obj.unwrap().expect_array::<jchar>();
    let arr_lock = arr.lock();

    // Reclaim elements so they get dropped at the end of the function
    Vec::from_raw_parts(chars as *mut jchar, arr_lock.len(), arr_lock.len());
}

#[no_mangle]
pub unsafe extern "system" fn NewStringUTF(env: *mut JNIEnv, utf: *const c_char) -> jstring {
    if env.is_null() {
        panic!("Got null for JNIEnv");
    }

    if utf.is_null() {
        return null_mut();
    }

    let input = CStr::from_ptr(utf);
    let env = RawJNIEnv::new(env);
    let mut jvm = env.write();
    jvm.build_string(&input.to_string_lossy())
        .expect_object()
        .ptr()
}

#[no_mangle]
pub unsafe extern "system" fn GetStringUTFLength(env: *mut JNIEnv, str: jstring) -> jsize {
    let env = RawJNIEnv::new(env);
    obj_expect!(env, str, 0).expect_string().len() as jsize
}

#[no_mangle]
pub unsafe extern "system" fn GetStringUTFChars(
    env: *mut JNIEnv,
    str: jstring,
    is_copy: *mut jboolean,
) -> *const c_char {
    let jvm = RawJNIEnv::new(env);
    if !is_copy.is_null() {
        *is_copy = JNI_TRUE;
    }

    let obj = obj_expect!(jvm, str, null());
    let str = obj.expect_string();

    let ret = CString::new(str).unwrap();
    ret.into_raw() as _
}

#[no_mangle]
pub unsafe extern "system" fn ReleaseStringUTFChars(
    env: *mut JNIEnv,
    str: jstring,
    chars: *const c_char,
) {
    // Put the pointer back into CString struct so it will be dropped at the end of the function
    drop(CString::from_raw(chars as _))
}

#[no_mangle]
pub unsafe extern "system" fn GetArrayLength(env: *mut JNIEnv, array: jarray) -> jsize {
    let env = RawJNIEnv::new(env);
    obj_expect!(env, array).unknown_array_length().unwrap() as jsize
}

macro_rules! impl_array {
    ($type:ty, $java_type:ty: $new_arr:ident, $get_elements:ident, $release_elements:ident, $get_region:ident, $set_region:ident) => {
        #[no_mangle]
        pub unsafe extern "system" fn $new_arr(_env: *mut JNIEnv, len: jsize) -> jarray {
            ObjectHandle::new_array::<$java_type>(len as usize).ptr()
        }

        #[no_mangle]
        pub unsafe extern "system" fn $get_elements(
            env: *mut JNIEnv,
            array: jarray,
            is_copy: *mut jboolean,
        ) -> *mut $type {
            let env = RawJNIEnv::new(env);
            if !is_copy.is_null() {
                *is_copy = JNI_TRUE;
            }
            let mut clone = obj_expect!(env, array, null_mut())
                .expect_array::<$java_type>()
                .lock()
                .to_vec();
            let ret = clone.as_mut_ptr();
            forget(clone); // Forget it so it can be recovered later
            ret as _
        }

        #[no_mangle]
        pub unsafe extern "system" fn $release_elements(
            env: *mut JNIEnv,
            array: jarray,
            elems: *mut $type,
            mode: jint,
        ) {
            let env = RawJNIEnv::new(env);
            let arr = obj_expect!(env, array).expect_array::<$java_type>();
            let mut lock = arr.lock();

            // Reclaim elements
            let elements = Vec::from_raw_parts(elems as *mut $java_type, lock.len(), lock.len());

            // Copy back elements
            if mode & JNI_ABORT == 0 {
                lock.copy_from_slice(&elements);
            }

            // Do not free the buffer
            if mode & JNI_COMMIT == 1 {
                forget(elements);
            }
        }

        #[no_mangle]
        pub unsafe extern "system" fn $get_region(
            env: *mut JNIEnv,
            array: jarray,
            start: jsize,
            len: jsize,
            buf: *mut $type,
        ) {
            let env = RawJNIEnv::new(env);
            let arr = obj_expect!(env, array).expect_array::<$java_type>();
            let lock = arr.lock();
            assert!(start >= 0 && len >= 0 && start + len <= lock.len() as jint);
            (lock.deref().as_ptr() as *const $type)
                .offset(start as isize)
                .copy_to(buf, len as usize);
        }

        #[no_mangle]
        pub unsafe extern "system" fn $set_region(
            env: *mut JNIEnv,
            array: jarray,
            start: jsize,
            len: jsize,
            buf: *const $type,
        ) {
            let env = RawJNIEnv::new(env);
            let arr = obj_expect!(env, array).expect_array::<$java_type>();
            let mut lock = arr.lock();
            assert!(start >= 0 && len >= 0 && start + len <= lock.len() as jint);
            buf.copy_to(
                (lock.deref_mut().as_mut_ptr() as *mut $type).offset(start as isize),
                len as usize,
            );
        }
    };
}

impl_array!(
    jboolean,
    jboolean: NewBooleanArray,
    GetBooleanArrayElements,
    ReleaseBooleanArrayElements,
    GetBooleanArrayRegion,
    SetBooleanArrayRegion
);
impl_array!(
    jbyte,
    jbyte: NewByteArray,
    GetByteArrayElements,
    ReleaseByteArrayElements,
    GetByteArrayRegion,
    SetByteArrayRegion
);
impl_array!(
    jchar,
    jchar: NewCharArray,
    GetCharArrayElements,
    ReleaseCharArrayElements,
    GetCharArrayRegion,
    SetCharArrayRegion
);
impl_array!(
    jshort,
    jshort: NewShortArray,
    GetShortArrayElements,
    ReleaseShortArrayElements,
    GetShortArrayRegion,
    SetShortArrayRegion
);
impl_array!(
    jint,
    jint: NewIntArray,
    GetIntArrayElements,
    ReleaseIntArrayElements,
    GetIntArrayRegion,
    SetIntArrayRegion
);
impl_array!(
    jlong,
    jlong: NewLongArray,
    GetLongArrayElements,
    ReleaseLongArrayElements,
    GetLongArrayRegion,
    SetLongArrayRegion
);
impl_array!(
    jfloat,
    jfloat: NewFloatArray,
    GetFloatArrayElements,
    ReleaseFloatArrayElements,
    GetFloatArrayRegion,
    SetFloatArrayRegion
);
impl_array!(
    jdouble,
    jdouble: NewDoubleArray,
    GetDoubleArrayElements,
    ReleaseDoubleArrayElements,
    GetDoubleArrayRegion,
    SetDoubleArrayRegion
);

#[no_mangle]
pub unsafe extern "system" fn NewObjectArray(
    _env: *mut JNIEnv,
    len: jsize,
    _clazz: jclass,
    init: jobject,
) -> jobjectArray {
    let initial = vec![ObjectHandle::from_ptr(init); len as usize];
    ObjectHandle::array_from_data(initial).ptr()
}
#[no_mangle]
pub unsafe extern "system" fn GetObjectArrayElement(
    env: *mut JNIEnv,
    array: jobjectArray,
    index: jsize,
) -> jobject {
    let env = RawJNIEnv::new(env);
    obj_expect!(env, array, null_mut())
        .expect_array::<Option<ObjectHandle>>()
        .lock()[index as usize]
        .pack()
        .l
}

#[no_mangle]
pub unsafe extern "system" fn SetObjectArrayElement(
    env: *mut JNIEnv,
    array: jobjectArray,
    index: jsize,
    val: jobject,
) {
    let env = RawJNIEnv::new(env);
    obj_expect!(env, array)
        .expect_array::<Option<ObjectHandle>>()
        .lock()
        .write_array(index as usize, ObjectHandle::from_ptr(val));
}

#[no_mangle]
pub unsafe extern "system" fn RegisterNatives(
    env: *mut JNIEnv,
    clazz: jclass,
    methods: *const JNINativeMethod,
    num_methods: jint,
) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterNatives(env: *mut JNIEnv, clazz: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn MonitorEnter(env: *mut JNIEnv, obj: jobject) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn MonitorExit(env: *mut JNIEnv, obj: jobject) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn GetJavaVM(env: *mut JNIEnv, vm: *mut *mut JavaVM) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn GetStringRegion(
    env: *mut JNIEnv,
    str: jstring,
    start: jsize,
    len: jsize,
    buf: *mut jchar,
) {
    let env = RawJNIEnv::new(env);
    let array: Option<ObjectHandle> = obj_expect!(env, str)
        .expect_instance()
        .lock()
        .read_named_field("value");
    let arr = array.unwrap().expect_array::<jchar>();
    let arr_lock = arr.lock();
    assert!(start >= 0 && len >= 0 && start + len <= arr_lock.len() as jint);
    (arr_lock.deref().as_ptr() as *const jchar)
        .offset(start as isize)
        .copy_to(buf, len as usize);
}

#[no_mangle]
pub unsafe extern "system" fn GetStringUTFRegion(
    env: *mut JNIEnv,
    str: jstring,
    start: jsize,
    len: jsize,
    buf: *mut c_char,
) {
    let env = RawJNIEnv::new(env);
    let str = obj_expect!(env, str).expect_string();

    copy_nonoverlapping(
        &str.as_bytes()[start as usize] as *const u8 as *const c_char,
        buf,
        len as usize,
    );
    write_bytes(buf.add(len as usize), 0, 1);
}

#[no_mangle]
pub unsafe extern "system" fn GetPrimitiveArrayCritical(
    env: *mut JNIEnv,
    array: jarray,
    is_copy: *mut jboolean,
) -> *mut c_void {
    let obj = obj_expect!(RawJNIEnv::new(env), array, null_mut()).memory_layout();
    match obj {
        ObjectType::Array(JavaTypeEnum::Boolean) => {
            (**env).GetBooleanArrayElements.unwrap()(env, array, is_copy) as _
        }
        ObjectType::Array(JavaTypeEnum::Byte) => {
            (**env).GetByteArrayElements.unwrap()(env, array, is_copy) as _
        }
        ObjectType::Array(JavaTypeEnum::Short) => {
            (**env).GetShortArrayElements.unwrap()(env, array, is_copy) as _
        }
        ObjectType::Array(JavaTypeEnum::Char) => {
            (**env).GetCharArrayElements.unwrap()(env, array, is_copy) as _
        }
        ObjectType::Array(JavaTypeEnum::Int) => {
            (**env).GetIntArrayElements.unwrap()(env, array, is_copy) as _
        }
        ObjectType::Array(JavaTypeEnum::Long) => {
            (**env).GetLongArrayElements.unwrap()(env, array, is_copy) as _
        }
        ObjectType::Array(JavaTypeEnum::Float) => {
            (**env).GetFloatArrayElements.unwrap()(env, array, is_copy) as _
        }
        ObjectType::Array(JavaTypeEnum::Double) => {
            (**env).GetDoubleArrayElements.unwrap()(env, array, is_copy) as _
        }
        _ => panic!(),
    }
}

#[no_mangle]
pub unsafe extern "system" fn ReleasePrimitiveArrayCritical(
    env: *mut JNIEnv,
    array: jarray,
    carray: *mut c_void,
    mode: jint,
) {
    let obj = obj_expect!(RawJNIEnv::new(env), array).memory_layout();
    match obj {
        ObjectType::Array(JavaTypeEnum::Boolean) => {
            (**env).ReleaseBooleanArrayElements.unwrap()(env, array, carray as _, mode)
        }
        ObjectType::Array(JavaTypeEnum::Byte) => {
            (**env).ReleaseByteArrayElements.unwrap()(env, array, carray as _, mode)
        }
        ObjectType::Array(JavaTypeEnum::Short) => {
            (**env).ReleaseShortArrayElements.unwrap()(env, array, carray as _, mode)
        }
        ObjectType::Array(JavaTypeEnum::Char) => {
            (**env).ReleaseCharArrayElements.unwrap()(env, array, carray as _, mode)
        }
        ObjectType::Array(JavaTypeEnum::Int) => {
            (**env).ReleaseIntArrayElements.unwrap()(env, array, carray as _, mode)
        }
        ObjectType::Array(JavaTypeEnum::Long) => {
            (**env).ReleaseLongArrayElements.unwrap()(env, array, carray as _, mode)
        }
        ObjectType::Array(JavaTypeEnum::Float) => {
            (**env).ReleaseFloatArrayElements.unwrap()(env, array, carray as _, mode)
        }
        ObjectType::Array(JavaTypeEnum::Double) => {
            (**env).ReleaseDoubleArrayElements.unwrap()(env, array, carray as _, mode)
        }
        _ => panic!(),
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetStringCritical(
    env: *mut JNIEnv,
    string: jstring,
    is_copy: *mut jboolean,
) -> *const jchar {
    (**env).GetStringChars.unwrap()(env, string, is_copy)
}

#[no_mangle]
pub unsafe extern "system" fn ReleaseStringCritical(
    env: *mut JNIEnv,
    string: jstring,
    cstring: *const jchar,
) {
    (**env).ReleaseStringChars.unwrap()(env, string, cstring)
}

#[no_mangle]
pub unsafe extern "system" fn NewWeakGlobalRef(env: *mut JNIEnv, obj: jobject) -> jweak {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn DeleteWeakGlobalRef(env: *mut JNIEnv, ref_: jweak) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn ExceptionCheck(env: *mut JNIEnv) -> jboolean {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn NewDirectByteBuffer(
    env: *mut JNIEnv,
    address: *mut c_void,
    capacity: jlong,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn GetDirectBufferAddress(
    env: *mut JNIEnv,
    buf: jobject,
) -> *mut c_void {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn GetDirectBufferCapacity(env: *mut JNIEnv, buf: jobject) -> jlong {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn GetObjectRefType(env: *mut JNIEnv, obj: jobject) -> jobjectRefType {
    unimplemented!()
}
