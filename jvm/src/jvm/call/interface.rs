use std::ffi::{CString, c_void};
use std::mem::forget;
use std::ptr::null_mut;

use jni::sys::{jclass, jint, JNIEnv, JNINativeInterface_, JNINativeMethod, jobject, jmethodID, jthrowable, jboolean, jvalue};

use crate::jvm::{ObjectHandle, JavaEnv};
use crate::jvm::mem::{ManualInstanceReference, ObjectReference, JavaValue, FieldDescriptor, JavaPrimitive};
use std::os::raw::c_char;
use crate::constant_pool::ClassElement;
use crate::class::BufferedRead;
use crate::jvm::call::FlowControl;

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

unsafe extern "system" fn get_method_id(env: *mut JNIEnv,
                                        clazz: jclass,
                                        name: *const c_char,
                                        sig: *const c_char)
                                        -> jmethodID {
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

unsafe extern "system" fn call_obj_method_a(env: *mut JNIEnv,
                                            obj: jobject,
                                            method_id: jmethodID,
                                            args: *const jvalue)
                                            -> jobject {
    let target = ObjectHandle::from_ptr(obj).unwrap();
    let element = read_method_id(method_id);
    let parsed_args = read_args(&element.desc, args);

    let mut jvm = &mut *((&**env).reserved0 as *mut JavaEnv);
    match jvm.invoke_virtual(element.clone(), target, parsed_args) {
        Ok(Some(JavaValue::Reference(v))) => v.pack().l,
        Err(FlowControl::Throws(x)) => {
            (&mut **(env as *mut *mut JNINativeInterface_)).reserved1 = x.pack().l as _;
            null_mut()
        },
        x => panic!("{:?}", x),
    }
}
// unsafe extern "C" fn call_method(env: *mut JNIEnv,
// obj: jobject,
// methodID: jmethodID,
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
        GetVersion: None,
        DefineClass: None,
        FindClass: None,
        FromReflectedMethod: None,
        FromReflectedField: None,
        ToReflectedMethod: None,
        GetSuperclass: None,
        IsAssignableFrom: None,
        ToReflectedField: None,
        Throw: None,
        ThrowNew: None,
        ExceptionOccurred: Some(exception_occurred),
        ExceptionDescribe: Some(exception_describe),
        ExceptionClear: Some(exception_clear),
        FatalError: None,
        PushLocalFrame: None,
        PopLocalFrame: None,
        NewGlobalRef: None,
        DeleteGlobalRef: None,
        DeleteLocalRef: None,
        IsSameObject: None,
        NewLocalRef: None,
        EnsureLocalCapacity: None,
        AllocObject: None,
        NewObject: None,
        NewObjectV: None,
        NewObjectA: None,
        GetObjectClass: Some(get_object_class),
        IsInstanceOf: None,
        GetMethodID: Some(get_method_id),
        CallObjectMethod: None,
        CallObjectMethodV: None,
        CallObjectMethodA: Some(call_obj_method_a),
        CallBooleanMethod: None,
        CallBooleanMethodV: None,
        CallBooleanMethodA: None,
        CallByteMethod: None,
        CallByteMethodV: None,
        CallByteMethodA: None,
        CallCharMethod: None,
        CallCharMethodV: None,
        CallCharMethodA: None,
        CallShortMethod: None,
        CallShortMethodV: None,
        CallShortMethodA: None,
        CallIntMethod: None,
        CallIntMethodV: None,
        CallIntMethodA: None,
        CallLongMethod: None,
        CallLongMethodV: None,
        CallLongMethodA: None,
        CallFloatMethod: None,
        CallFloatMethodV: None,
        CallFloatMethodA: None,
        CallDoubleMethod: None,
        CallDoubleMethodV: None,
        CallDoubleMethodA: None,
        CallVoidMethod: None,
        CallVoidMethodV: None,
        CallVoidMethodA: None,
        CallNonvirtualObjectMethod: None,
        CallNonvirtualObjectMethodV: None,
        CallNonvirtualObjectMethodA: None,
        CallNonvirtualBooleanMethod: None,
        CallNonvirtualBooleanMethodV: None,
        CallNonvirtualBooleanMethodA: None,
        CallNonvirtualByteMethod: None,
        CallNonvirtualByteMethodV: None,
        CallNonvirtualByteMethodA: None,
        CallNonvirtualCharMethod: None,
        CallNonvirtualCharMethodV: None,
        CallNonvirtualCharMethodA: None,
        CallNonvirtualShortMethod: None,
        CallNonvirtualShortMethodV: None,
        CallNonvirtualShortMethodA: None,
        CallNonvirtualIntMethod: None,
        CallNonvirtualIntMethodV: None,
        CallNonvirtualIntMethodA: None,
        CallNonvirtualLongMethod: None,
        CallNonvirtualLongMethodV: None,
        CallNonvirtualLongMethodA: None,
        CallNonvirtualFloatMethod: None,
        CallNonvirtualFloatMethodV: None,
        CallNonvirtualFloatMethodA: None,
        CallNonvirtualDoubleMethod: None,
        CallNonvirtualDoubleMethodV: None,
        CallNonvirtualDoubleMethodA: None,
        CallNonvirtualVoidMethod: None,
        CallNonvirtualVoidMethodV: None,
        CallNonvirtualVoidMethodA: None,
        GetFieldID: None,
        GetObjectField: None,
        GetBooleanField: None,
        GetByteField: None,
        GetCharField: None,
        GetShortField: None,
        GetIntField: None,
        GetLongField: None,
        GetFloatField: None,
        GetDoubleField: None,
        SetObjectField: None,
        SetBooleanField: None,
        SetByteField: None,
        SetCharField: None,
        SetShortField: None,
        SetIntField: None,
        SetLongField: None,
        SetFloatField: None,
        SetDoubleField: None,
        GetStaticMethodID: None,
        CallStaticObjectMethod: None,
        CallStaticObjectMethodV: None,
        CallStaticObjectMethodA: None,
        CallStaticBooleanMethod: None,
        CallStaticBooleanMethodV: None,
        CallStaticBooleanMethodA: None,
        CallStaticByteMethod: None,
        CallStaticByteMethodV: None,
        CallStaticByteMethodA: None,
        CallStaticCharMethod: None,
        CallStaticCharMethodV: None,
        CallStaticCharMethodA: None,
        CallStaticShortMethod: None,
        CallStaticShortMethodV: None,
        CallStaticShortMethodA: None,
        CallStaticIntMethod: None,
        CallStaticIntMethodV: None,
        CallStaticIntMethodA: None,
        CallStaticLongMethod: None,
        CallStaticLongMethodV: None,
        CallStaticLongMethodA: None,
        CallStaticFloatMethod: None,
        CallStaticFloatMethodV: None,
        CallStaticFloatMethodA: None,
        CallStaticDoubleMethod: None,
        CallStaticDoubleMethodV: None,
        CallStaticDoubleMethodA: None,
        CallStaticVoidMethod: None,
        CallStaticVoidMethodV: None,
        CallStaticVoidMethodA: None,
        GetStaticFieldID: None,
        GetStaticObjectField: None,
        GetStaticBooleanField: None,
        GetStaticByteField: None,
        GetStaticCharField: None,
        GetStaticShortField: None,
        GetStaticIntField: None,
        GetStaticLongField: None,
        GetStaticFloatField: None,
        GetStaticDoubleField: None,
        SetStaticObjectField: None,
        SetStaticBooleanField: None,
        SetStaticByteField: None,
        SetStaticCharField: None,
        SetStaticShortField: None,
        SetStaticIntField: None,
        SetStaticLongField: None,
        SetStaticFloatField: None,
        SetStaticDoubleField: None,
        NewString: None,
        GetStringLength: None,
        GetStringChars: None,
        ReleaseStringChars: None,
        NewStringUTF: None,
        GetStringUTFLength: None,
        GetStringUTFChars: None,
        ReleaseStringUTFChars: None,
        GetArrayLength: None,
        NewObjectArray: None,
        GetObjectArrayElement: None,
        SetObjectArrayElement: None,
        NewBooleanArray: None,
        NewByteArray: None,
        NewCharArray: None,
        NewShortArray: None,
        NewIntArray: None,
        NewLongArray: None,
        NewFloatArray: None,
        NewDoubleArray: None,
        GetBooleanArrayElements: None,
        GetByteArrayElements: None,
        GetCharArrayElements: None,
        GetShortArrayElements: None,
        GetIntArrayElements: None,
        GetLongArrayElements: None,
        GetFloatArrayElements: None,
        GetDoubleArrayElements: None,
        ReleaseBooleanArrayElements: None,
        ReleaseByteArrayElements: None,
        ReleaseCharArrayElements: None,
        ReleaseShortArrayElements: None,
        ReleaseIntArrayElements: None,
        ReleaseLongArrayElements: None,
        ReleaseFloatArrayElements: None,
        ReleaseDoubleArrayElements: None,
        GetBooleanArrayRegion: None,
        GetByteArrayRegion: None,
        GetCharArrayRegion: None,
        GetShortArrayRegion: None,
        GetIntArrayRegion: None,
        GetLongArrayRegion: None,
        GetFloatArrayRegion: None,
        GetDoubleArrayRegion: None,
        SetBooleanArrayRegion: None,
        SetByteArrayRegion: None,
        SetCharArrayRegion: None,
        SetShortArrayRegion: None,
        SetIntArrayRegion: None,
        SetLongArrayRegion: None,
        SetFloatArrayRegion: None,
        SetDoubleArrayRegion: None,
        RegisterNatives: Some(register_natives),
        UnregisterNatives: None,
        MonitorEnter: None,
        MonitorExit: None,
        GetJavaVM: None,
        GetStringRegion: None,
        GetStringUTFRegion: None,
        GetPrimitiveArrayCritical: None,
        ReleasePrimitiveArrayCritical: None,
        GetStringCritical: None,
        ReleaseStringCritical: None,
        NewWeakGlobalRef: None,
        DeleteWeakGlobalRef: None,
        ExceptionCheck: Some(exception_check),
        NewDirectByteBuffer: None,
        GetDirectBufferAddress: None,
        GetDirectBufferCapacity: None,
        GetObjectRefType: None,
    }
}
