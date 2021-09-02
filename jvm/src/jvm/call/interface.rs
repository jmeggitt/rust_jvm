#![allow(unused_variables)]

use std::ffi::{c_void, CString};
use std::mem::forget;
use std::ptr::null_mut;

use jni::sys::{
    jarray, jboolean, jbooleanArray, jbyte, jbyteArray, jchar, jcharArray, jclass, jdouble,
    jdoubleArray, jfield_id, jfloat, jfloatArray, jint, jintArray, jlong, jlongArray, jmethod_id,
    jobject, jobjectArray, jobjectRefType, jshort, jshortArray, jsize, jstring, jthrowable, jvalue,
    jweak, va_list, JNIEnv, JNINativeInterface_, JNINativeMethod, JavaVM,
};

use crate::class::BufferedRead;
use crate::constant_pool::ClassElement;
use crate::jvm::call::FlowControl;
use crate::jvm::mem::{
    FieldDescriptor, JavaPrimitive, JavaValue, ManualInstanceReference, ObjectReference,
};
use crate::jvm::{JavaEnv, ObjectHandle};
use std::os::raw::c_char;

// #[deprecated(note="Switch to storing JVM ptr in JNIEnv::reserved0")]
// pub static mut GLOBAL_JVM: Option<Box<JavaEnv>> = None;

pub unsafe extern "system" fn register_natives(
    _env: *mut JNIEnv,
    clazz: jclass,
    methods: *const JNINativeMethod,
    num_methods: jint,
) -> jint {
    debug!("Calling JNIEnv::RegisterNatives");
    let a = ObjectHandle::from_ptr(clazz).unwrap().expect_instance();
    let name_obj: Option<ObjectHandle> = a.read_named_field("name");
    let class = name_obj.unwrap().expect_string();
    // let class_object = ObjectHandle::from_ptr(clazz).unwrap();
    // let class = class_object.expect_string();

    let mut registered = 0;

    for method in
        std::slice::from_raw_parts_mut(methods as *mut JNINativeMethod, num_methods as usize)
    {
        let name = CString::from_raw(method.name);
        let desc = CString::from_raw(method.signature);

        let jvm = (&**_env).reserved0 as *mut JavaEnv;
        if (&mut *jvm).linked_libraries.register_fn(
            &class,
            name.to_str().unwrap(),
            desc.to_str().unwrap(),
            method.fnPtr,
        ) {
            registered += 1;
        }

        forget(name);
        forget(desc);
    }

    forget(class);
    registered
}

unsafe extern "system" fn get_object_class(env: *mut JNIEnv, obj: jobject) -> jclass {
    let jvm = (&**env).reserved0 as *mut JavaEnv;
    match ObjectHandle::from_ptr(obj) {
        Some(v) => (&mut *jvm).class_instance(&v.get_class()).ptr(),
        None => null_mut(),
    }
}

unsafe extern "system" fn get_method_id(
    env: *mut JNIEnv,
    clazz: jclass,
    name: *const c_char,
    sig: *const c_char,
) -> jmethod_id {
    let a = ObjectHandle::from_ptr(clazz).unwrap().expect_instance();
    let name_obj: Option<ObjectHandle> = a.read_named_field("name");

    let name_string = CString::from_raw(name as *mut _);
    let desc_string = CString::from_raw(sig as *mut _);

    let element = ClassElement {
        class: name_obj.unwrap().expect_string(),
        element: name_string.to_str().unwrap().to_string(),
        desc: desc_string.to_str().unwrap().to_string(),
    };

    forget(name_string);
    forget(desc_string);

    Box::leak(Box::new(element)) as *mut ClassElement as *mut _
}

unsafe extern "system" fn exception_check(env: *mut JNIEnv) -> jboolean {
    !(&**env).reserved1.is_null() as jboolean
}

unsafe extern "system" fn exception_clear(env: *mut JNIEnv) {
    (&mut **(env as *mut *mut JNINativeInterface_)).reserved1 = null_mut();
}

unsafe extern "system" fn exception_occurred(env: *mut JNIEnv) -> jthrowable {
    (&mut **(env as *mut *mut JNINativeInterface_)).reserved1 as _
}

unsafe extern "system" fn exception_describe(env: *mut JNIEnv) {
    eprintln!("*descriptive explanation of exception*")
}

unsafe fn read_method_id(method: jmethod_id) -> &'static ClassElement {
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
    method_id: jmethod_id,
    args: *const jvalue,
) -> jobject {
    let target = ObjectHandle::from_ptr(obj).unwrap();
    let element = read_method_id(method_id);
    let parsed_args = read_args(&element.desc, args);

    let mut jvm = &mut *((&**env).reserved0 as *mut JavaEnv);
    match jvm.invoke_virtual(element.clone(), target, parsed_args) {
        Ok(Some(JavaValue::Reference(v))) => v.pack().l,
        Err(FlowControl::Throws(x)) => {
            (&mut **(env as *mut *mut JNINativeInterface_)).reserved1 = x.pack().l as _;
            null_mut()
        }
        x => panic!("{:?}", x),
    }
}
// unsafe extern "C" fn call_method(env: *mut JNIEnv,
// obj: jobject,
// method_id: jmethod_id,
// ...)
// -> jobject {
//
// }

#[inline]
pub fn build_interface(jvm: &mut JavaEnv) -> JNINativeInterface_ {
    JNINativeInterface_ {
        reserved0: jvm as *mut JavaEnv as *mut c_void,
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
        CallObjectMethod: None,
        CallObjectMethodV: Some(CallObjectMethodV),
        CallObjectMethodA: Some(call_obj_method_a),
        CallBooleanMethod: None,
        CallBooleanMethodV: Some(CallBooleanMethodV),
        CallBooleanMethodA: Some(CallBooleanMethodA),
        CallByteMethod: None,
        CallByteMethodV: Some(CallByteMethodV),
        CallByteMethodA: Some(CallByteMethodA),
        CallCharMethod: None,
        CallCharMethodV: Some(CallCharMethodV),
        CallCharMethodA: Some(CallCharMethodA),
        CallShortMethod: None,
        CallShortMethodV: Some(CallShortMethodV),
        CallShortMethodA: Some(CallShortMethodA),
        CallIntMethod: None,
        CallIntMethodV: Some(CallIntMethodV),
        CallIntMethodA: Some(CallIntMethodA),
        CallLongMethod: None,
        CallLongMethodV: Some(CallLongMethodV),
        CallLongMethodA: Some(CallLongMethodA),
        CallFloatMethod: None,
        CallFloatMethodV: Some(CallFloatMethodV),
        CallFloatMethodA: Some(CallFloatMethodA),
        CallDoubleMethod: None,
        CallDoubleMethodV: Some(CallDoubleMethodV),
        CallDoubleMethodA: Some(CallDoubleMethodA),
        CallVoidMethod: None,
        CallVoidMethodV: Some(CallVoidMethodV),
        CallVoidMethodA: Some(CallVoidMethodA),
        CallNonvirtualObjectMethod: None,
        CallNonvirtualObjectMethodV: Some(CallNonvirtualObjectMethodV),
        CallNonvirtualObjectMethodA: Some(CallNonvirtualObjectMethodA),
        CallNonvirtualBooleanMethod: None,
        CallNonvirtualBooleanMethodV: Some(CallNonvirtualBooleanMethodV),
        CallNonvirtualBooleanMethodA: Some(CallNonvirtualBooleanMethodA),
        CallNonvirtualByteMethod: None,
        CallNonvirtualByteMethodV: Some(CallNonvirtualByteMethodV),
        CallNonvirtualByteMethodA: Some(CallNonvirtualByteMethodA),
        CallNonvirtualCharMethod: None,
        CallNonvirtualCharMethodV: Some(CallNonvirtualCharMethodV),
        CallNonvirtualCharMethodA: Some(CallNonvirtualCharMethodA),
        CallNonvirtualShortMethod: None,
        CallNonvirtualShortMethodV: Some(CallNonvirtualShortMethodV),
        CallNonvirtualShortMethodA: Some(CallNonvirtualShortMethodA),
        CallNonvirtualIntMethod: None,
        CallNonvirtualIntMethodV: Some(CallNonvirtualIntMethodV),
        CallNonvirtualIntMethodA: Some(CallNonvirtualIntMethodA),
        CallNonvirtualLongMethod: None,
        CallNonvirtualLongMethodV: Some(CallNonvirtualLongMethodV),
        CallNonvirtualLongMethodA: Some(CallNonvirtualLongMethodA),
        CallNonvirtualFloatMethod: None,
        CallNonvirtualFloatMethodV: Some(CallNonvirtualFloatMethodV),
        CallNonvirtualFloatMethodA: Some(CallNonvirtualFloatMethodA),
        CallNonvirtualDoubleMethod: None,
        CallNonvirtualDoubleMethodV: Some(CallNonvirtualDoubleMethodV),
        CallNonvirtualDoubleMethodA: Some(CallNonvirtualDoubleMethodA),
        CallNonvirtualVoidMethod: None,
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
        CallStaticObjectMethod: None,
        CallStaticObjectMethodV: Some(CallStaticObjectMethodV),
        CallStaticObjectMethodA: Some(CallStaticObjectMethodA),
        CallStaticBooleanMethod: None,
        CallStaticBooleanMethodV: Some(CallStaticBooleanMethodV),
        CallStaticBooleanMethodA: Some(CallStaticBooleanMethodA),
        CallStaticByteMethod: None,
        CallStaticByteMethodV: Some(CallStaticByteMethodV),
        CallStaticByteMethodA: Some(CallStaticByteMethodA),
        CallStaticCharMethod: None,
        CallStaticCharMethodV: Some(CallStaticCharMethodV),
        CallStaticCharMethodA: Some(CallStaticCharMethodA),
        CallStaticShortMethod: None,
        CallStaticShortMethodV: Some(CallStaticShortMethodV),
        CallStaticShortMethodA: Some(CallStaticShortMethodA),
        CallStaticIntMethod: None,
        CallStaticIntMethodV: Some(CallStaticIntMethodV),
        CallStaticIntMethodA: Some(CallStaticIntMethodA),
        CallStaticLongMethod: None,
        CallStaticLongMethodV: Some(CallStaticLongMethodV),
        CallStaticLongMethodA: Some(CallStaticLongMethodA),
        CallStaticFloatMethod: None,
        CallStaticFloatMethodV: Some(CallStaticFloatMethodV),
        CallStaticFloatMethodA: Some(CallStaticFloatMethodA),
        CallStaticDoubleMethod: None,
        CallStaticDoubleMethodV: Some(CallStaticDoubleMethodV),
        CallStaticDoubleMethodA: Some(CallStaticDoubleMethodA),
        CallStaticVoidMethod: None,
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
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn FromReflectedMethod(env: *mut JNIEnv, method: jobject) -> jmethod_id {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn FromReflectedField(env: *mut JNIEnv, field: jobject) -> jfield_id {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ToReflectedMethod(
    env: *mut JNIEnv,
    cls: jclass,
    method_id: jmethod_id,
    is_static: jboolean,
) -> jobject {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetSuperclass(env: *mut JNIEnv, sub: jclass) -> jclass {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn IsAssignableFrom(
    env: *mut JNIEnv,
    sub: jclass,
    sup: jclass,
) -> jboolean {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ToReflectedField(
    env: *mut JNIEnv,
    cls: jclass,
    field_id: jfield_id,
    is_static: jboolean,
) -> jobject {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn Throw(env: *mut JNIEnv, obj: jthrowable) -> jint {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ThrowNew(
    env: *mut JNIEnv,
    clazz: jclass,
    msg: *const c_char,
) -> jint {
    unimplemented!()
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
    unimplemented!()
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
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn DeleteGlobalRef(env: *mut JNIEnv, gref: jobject) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn DeleteLocalRef(env: *mut JNIEnv, obj: jobject) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn IsSameObject(
    env: *mut JNIEnv,
    obj1: jobject,
    obj2: jobject,
) -> jboolean {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn NewLocalRef(env: *mut JNIEnv, ref_: jobject) -> jobject {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn EnsureLocalCapacity(env: *mut JNIEnv, capacity: jint) -> jint {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn AllocObject(env: *mut JNIEnv, clazz: jclass) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn NewObjectV(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jobject {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn NewObjectA(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jobject {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetObjectClass(env: *mut JNIEnv, obj: jobject) -> jclass {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn IsInstanceOf(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
) -> jboolean {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn CallObjectMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: va_list,
) -> jobject {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallObjectMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn CallBooleanMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: va_list,
) -> jboolean {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallBooleanMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jboolean {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn CallByteMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: va_list,
) -> jbyte {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallByteMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jbyte {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn CallCharMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: va_list,
) -> jchar {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallCharMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jchar {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn CallShortMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: va_list,
) -> jshort {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallShortMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jshort {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn CallIntMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: va_list,
) -> jint {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallIntMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn CallLongMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: va_list,
) -> jlong {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallLongMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jlong {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn CallFloatMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: va_list,
) -> jfloat {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallFloatMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jfloat {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallDoubleMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: va_list,
) -> jdouble {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallDoubleMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jdouble {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallVoidMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: va_list,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallVoidMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    method_id: jmethod_id,
    args: *const jvalue,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualObjectMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jobject {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualObjectMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jobject {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualBooleanMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jboolean {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualBooleanMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jboolean {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualByteMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jbyte {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualByteMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jbyte {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualCharMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jchar {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualCharMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jchar {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualShortMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jshort {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualShortMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jshort {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualIntMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jint {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualIntMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jint {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualLongMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jlong {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualLongMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jlong {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualFloatMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jfloat {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualFloatMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jfloat {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualDoubleMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jdouble {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualDoubleMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jdouble {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualVoidMethodV(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallNonvirtualVoidMethodA(
    env: *mut JNIEnv,
    obj: jobject,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetFieldID(
    env: *mut JNIEnv,
    clazz: jclass,
    name: *const c_char,
    sig: *const c_char,
) -> jfield_id {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetObjectField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
) -> jobject {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetBooleanField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
) -> jboolean {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetByteField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
) -> jbyte {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetCharField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
) -> jchar {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetShortField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
) -> jshort {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetIntField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
) -> jint {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetLongField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
) -> jlong {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetFloatField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
) -> jfloat {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetDoubleField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
) -> jdouble {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetObjectField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
    val: jobject,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetBooleanField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
    val: jboolean,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetByteField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
    val: jbyte,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetCharField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
    val: jchar,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetShortField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
    val: jshort,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetIntField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
    val: jint,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetLongField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
    val: jlong,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetFloatField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
    val: jfloat,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetDoubleField(
    env: *mut JNIEnv,
    obj: jobject,
    field_id: jfield_id,
    val: jdouble,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStaticMethodID(
    env: *mut JNIEnv,
    clazz: jclass,
    name: *const c_char,
    sig: *const c_char,
) -> jmethod_id {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticObjectMethodV(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jobject {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticObjectMethodA(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jobject {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticBooleanMethodV(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jboolean {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticBooleanMethodA(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jboolean {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticByteMethodV(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jbyte {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticByteMethodA(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jbyte {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticCharMethodV(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jchar {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticCharMethodA(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jchar {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticShortMethodV(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jshort {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticShortMethodA(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jshort {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticIntMethodV(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jint {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticIntMethodA(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jint {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticLongMethodV(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jlong {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticLongMethodA(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jlong {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticFloatMethodV(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jfloat {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticFloatMethodA(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jfloat {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticDoubleMethodV(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: va_list,
) -> jdouble {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticDoubleMethodA(
    env: *mut JNIEnv,
    clazz: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) -> jdouble {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticVoidMethodV(
    env: *mut JNIEnv,
    cls: jclass,
    method_id: jmethod_id,
    args: va_list,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn CallStaticVoidMethodA(
    env: *mut JNIEnv,
    cls: jclass,
    method_id: jmethod_id,
    args: *const jvalue,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStaticFieldID(
    env: *mut JNIEnv,
    clazz: jclass,
    name: *const c_char,
    sig: *const c_char,
) -> jfield_id {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStaticObjectField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
) -> jobject {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStaticBooleanField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
) -> jboolean {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStaticByteField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
) -> jbyte {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStaticCharField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
) -> jchar {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStaticShortField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
) -> jshort {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStaticIntField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
) -> jint {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStaticLongField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
) -> jlong {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStaticFloatField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
) -> jfloat {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStaticDoubleField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
) -> jdouble {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetStaticObjectField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
    value: jobject,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetStaticBooleanField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
    value: jboolean,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetStaticByteField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
    value: jbyte,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetStaticCharField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
    value: jchar,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetStaticShortField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
    value: jshort,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetStaticIntField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
    value: jint,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetStaticLongField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
    value: jlong,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetStaticFloatField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
    value: jfloat,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetStaticDoubleField(
    env: *mut JNIEnv,
    clazz: jclass,
    field_id: jfield_id,
    value: jdouble,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn NewString(
    env: *mut JNIEnv,
    unicode: *const jchar,
    len: jsize,
) -> jstring {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStringLength(env: *mut JNIEnv, str: jstring) -> jsize {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStringChars(
    env: *mut JNIEnv,
    str: jstring,
    is_copy: *mut jboolean,
) -> *const jchar {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ReleaseStringChars(
    env: *mut JNIEnv,
    str: jstring,
    chars: *const jchar,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn NewStringUTF(env: *mut JNIEnv, utf: *const c_char) -> jstring {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStringUTFLength(env: *mut JNIEnv, str: jstring) -> jsize {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStringUTFChars(
    env: *mut JNIEnv,
    str: jstring,
    is_copy: *mut jboolean,
) -> *const c_char {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ReleaseStringUTFChars(
    env: *mut JNIEnv,
    str: jstring,
    chars: *const c_char,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetArrayLength(env: *mut JNIEnv, array: jarray) -> jsize {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn NewObjectArray(
    env: *mut JNIEnv,
    len: jsize,
    clazz: jclass,
    init: jobject,
) -> jobjectArray {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetObjectArrayElement(
    env: *mut JNIEnv,
    array: jobjectArray,
    index: jsize,
) -> jobject {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetObjectArrayElement(
    env: *mut JNIEnv,
    array: jobjectArray,
    index: jsize,
    val: jobject,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn NewBooleanArray(env: *mut JNIEnv, len: jsize) -> jbooleanArray {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn NewByteArray(env: *mut JNIEnv, len: jsize) -> jbyteArray {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn NewCharArray(env: *mut JNIEnv, len: jsize) -> jcharArray {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn NewShortArray(env: *mut JNIEnv, len: jsize) -> jshortArray {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn NewIntArray(env: *mut JNIEnv, len: jsize) -> jintArray {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn NewLongArray(env: *mut JNIEnv, len: jsize) -> jlongArray {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn NewFloatArray(env: *mut JNIEnv, len: jsize) -> jfloatArray {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn NewDoubleArray(env: *mut JNIEnv, len: jsize) -> jdoubleArray {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetBooleanArrayElements(
    env: *mut JNIEnv,
    array: jbooleanArray,
    is_copy: *mut jboolean,
) -> *mut jboolean {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetByteArrayElements(
    env: *mut JNIEnv,
    array: jbyteArray,
    is_copy: *mut jboolean,
) -> *mut jbyte {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetCharArrayElements(
    env: *mut JNIEnv,
    array: jcharArray,
    is_copy: *mut jboolean,
) -> *mut jchar {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetShortArrayElements(
    env: *mut JNIEnv,
    array: jshortArray,
    is_copy: *mut jboolean,
) -> *mut jshort {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetIntArrayElements(
    env: *mut JNIEnv,
    array: jintArray,
    is_copy: *mut jboolean,
) -> *mut jint {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetLongArrayElements(
    env: *mut JNIEnv,
    array: jlongArray,
    is_copy: *mut jboolean,
) -> *mut jlong {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetFloatArrayElements(
    env: *mut JNIEnv,
    array: jfloatArray,
    is_copy: *mut jboolean,
) -> *mut jfloat {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetDoubleArrayElements(
    env: *mut JNIEnv,
    array: jdoubleArray,
    is_copy: *mut jboolean,
) -> *mut jdouble {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ReleaseBooleanArrayElements(
    env: *mut JNIEnv,
    array: jbooleanArray,
    elems: *mut jboolean,
    mode: jint,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ReleaseByteArrayElements(
    env: *mut JNIEnv,
    array: jbyteArray,
    elems: *mut jbyte,
    mode: jint,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ReleaseCharArrayElements(
    env: *mut JNIEnv,
    array: jcharArray,
    elems: *mut jchar,
    mode: jint,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ReleaseShortArrayElements(
    env: *mut JNIEnv,
    array: jshortArray,
    elems: *mut jshort,
    mode: jint,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ReleaseIntArrayElements(
    env: *mut JNIEnv,
    array: jintArray,
    elems: *mut jint,
    mode: jint,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ReleaseLongArrayElements(
    env: *mut JNIEnv,
    array: jlongArray,
    elems: *mut jlong,
    mode: jint,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ReleaseFloatArrayElements(
    env: *mut JNIEnv,
    array: jfloatArray,
    elems: *mut jfloat,
    mode: jint,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ReleaseDoubleArrayElements(
    env: *mut JNIEnv,
    array: jdoubleArray,
    elems: *mut jdouble,
    mode: jint,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetBooleanArrayRegion(
    env: *mut JNIEnv,
    array: jbooleanArray,
    start: jsize,
    l: jsize,
    buf: *mut jboolean,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetByteArrayRegion(
    env: *mut JNIEnv,
    array: jbyteArray,
    start: jsize,
    len: jsize,
    buf: *mut jbyte,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetCharArrayRegion(
    env: *mut JNIEnv,
    array: jcharArray,
    start: jsize,
    len: jsize,
    buf: *mut jchar,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetShortArrayRegion(
    env: *mut JNIEnv,
    array: jshortArray,
    start: jsize,
    len: jsize,
    buf: *mut jshort,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetIntArrayRegion(
    env: *mut JNIEnv,
    array: jintArray,
    start: jsize,
    len: jsize,
    buf: *mut jint,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetLongArrayRegion(
    env: *mut JNIEnv,
    array: jlongArray,
    start: jsize,
    len: jsize,
    buf: *mut jlong,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetFloatArrayRegion(
    env: *mut JNIEnv,
    array: jfloatArray,
    start: jsize,
    len: jsize,
    buf: *mut jfloat,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetDoubleArrayRegion(
    env: *mut JNIEnv,
    array: jdoubleArray,
    start: jsize,
    len: jsize,
    buf: *mut jdouble,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetBooleanArrayRegion(
    env: *mut JNIEnv,
    array: jbooleanArray,
    start: jsize,
    l: jsize,
    buf: *const jboolean,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetByteArrayRegion(
    env: *mut JNIEnv,
    array: jbyteArray,
    start: jsize,
    len: jsize,
    buf: *const jbyte,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetCharArrayRegion(
    env: *mut JNIEnv,
    array: jcharArray,
    start: jsize,
    len: jsize,
    buf: *const jchar,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetShortArrayRegion(
    env: *mut JNIEnv,
    array: jshortArray,
    start: jsize,
    len: jsize,
    buf: *const jshort,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetIntArrayRegion(
    env: *mut JNIEnv,
    array: jintArray,
    start: jsize,
    len: jsize,
    buf: *const jint,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetLongArrayRegion(
    env: *mut JNIEnv,
    array: jlongArray,
    start: jsize,
    len: jsize,
    buf: *const jlong,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetFloatArrayRegion(
    env: *mut JNIEnv,
    array: jfloatArray,
    start: jsize,
    len: jsize,
    buf: *const jfloat,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn SetDoubleArrayRegion(
    env: *mut JNIEnv,
    array: jdoubleArray,
    start: jsize,
    len: jsize,
    buf: *const jdouble,
) {
    unimplemented!()
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
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStringUTFRegion(
    env: *mut JNIEnv,
    str: jstring,
    start: jsize,
    len: jsize,
    buf: *mut c_char,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetPrimitiveArrayCritical(
    env: *mut JNIEnv,
    array: jarray,
    is_copy: *mut jboolean,
) -> *mut c_void {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ReleasePrimitiveArrayCritical(
    env: *mut JNIEnv,
    array: jarray,
    carray: *mut c_void,
    mode: jint,
) {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn GetStringCritical(
    env: *mut JNIEnv,
    string: jstring,
    is_copy: *mut jboolean,
) -> *const jchar {
    unimplemented!()
}
#[no_mangle]
pub unsafe extern "system" fn ReleaseStringCritical(
    env: *mut JNIEnv,
    string: jstring,
    cstring: *const jchar,
) {
    unimplemented!()
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
