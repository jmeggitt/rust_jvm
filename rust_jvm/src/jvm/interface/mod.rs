use jni::sys::{
    jarray, jboolean, jbooleanArray, jbyte, jbyteArray, jchar, jcharArray, jclass, jdouble,
    jdoubleArray, jfieldID, jfloat, jfloatArray, jint, jintArray, jlong, jlongArray, jmethodID,
    jobject, jobjectArray, jobjectRefType, jshort, jshortArray, jsize, jstring, jthrowable, jvalue,
    jweak, va_list, JNINativeMethod, JavaVM,
};
use libc::c_char;
use std::ffi::c_void;

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct JNINativeInterface {
    pub reserved0: *mut c_void,
    pub reserved1: *mut c_void,
    pub reserved2: *mut c_void,
    pub reserved3: *mut c_void,
    pub GetVersion: extern "system" fn(env: *mut JNIEnv) -> jint,
    pub DefineClass: extern "system" fn(
        env: *mut JNIEnv,
        name: *const c_char,
        loader: jobject,
        buf: *const jbyte,
        len: jsize,
    ) -> jclass,
    pub FindClass: extern "system" fn(env: *mut JNIEnv, name: *const c_char) -> jclass,
    pub FromReflectedMethod: extern "system" fn(env: *mut JNIEnv, method: jobject) -> jmethodID,
    pub FromReflectedField: extern "system" fn(env: *mut JNIEnv, field: jobject) -> jfieldID,
    pub ToReflectedMethod: extern "system" fn(
        env: *mut JNIEnv,
        cls: jclass,
        methodID: jmethodID,
        isStatic: jboolean,
    ) -> jobject,
    pub GetSuperclass: extern "system" fn(env: *mut JNIEnv, sub: jclass) -> jclass,
    pub IsAssignableFrom:
        extern "system" fn(env: *mut JNIEnv, sub: jclass, sup: jclass) -> jboolean,
    pub ToReflectedField: extern "system" fn(
        env: *mut JNIEnv,
        cls: jclass,
        fieldID: jfieldID,
        isStatic: jboolean,
    ) -> jobject,
    pub Throw: extern "system" fn(env: *mut JNIEnv, obj: jthrowable) -> jint,
    pub ThrowNew: extern "system" fn(env: *mut JNIEnv, clazz: jclass, msg: *const c_char) -> jint,
    pub ExceptionOccurred: extern "system" fn(env: *mut JNIEnv) -> jthrowable,
    pub ExceptionDescribe: extern "system" fn(env: *mut JNIEnv),
    pub ExceptionClear: extern "system" fn(env: *mut JNIEnv),
    pub FatalError: extern "system" fn(env: *mut JNIEnv, msg: *const c_char) -> !,
    pub PushLocalFrame: extern "system" fn(env: *mut JNIEnv, capacity: jint) -> jint,
    pub PopLocalFrame: extern "system" fn(env: *mut JNIEnv, result: jobject) -> jobject,
    pub NewGlobalRef: extern "system" fn(env: *mut JNIEnv, lobj: jobject) -> jobject,
    pub DeleteGlobalRef: extern "system" fn(env: *mut JNIEnv, gref: jobject),
    pub DeleteLocalRef: extern "system" fn(env: *mut JNIEnv, obj: jobject),
    pub IsSameObject:
        extern "system" fn(env: *mut JNIEnv, obj1: jobject, obj2: jobject) -> jboolean,
    pub NewLocalRef: extern "system" fn(env: *mut JNIEnv, ref_: jobject) -> jobject,
    pub EnsureLocalCapacity: extern "system" fn(env: *mut JNIEnv, capacity: jint) -> jint,
    pub AllocObject: extern "system" fn(env: *mut JNIEnv, clazz: jclass) -> jobject,
    pub NewObject:
        extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jobject,
    pub NewObjectV: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jobject,
    pub NewObjectA: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jobject,
    pub GetObjectClass: extern "system" fn(env: *mut JNIEnv, obj: jobject) -> jclass,
    pub IsInstanceOf: extern "system" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass) -> jboolean,
    pub GetMethodID: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        name: *const c_char,
        sig: *const c_char,
    ) -> jmethodID,
    pub CallObjectMethod:
        extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jobject,
    pub CallObjectMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: va_list,
    ) -> jobject,
    pub CallObjectMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jobject,
    pub CallBooleanMethod:
        extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jboolean,
    pub CallBooleanMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: va_list,
    ) -> jboolean,
    pub CallBooleanMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jboolean,
    pub CallByteMethod:
        extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jbyte,
    pub CallByteMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: va_list,
    ) -> jbyte,
    pub CallByteMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jbyte,
    pub CallCharMethod:
        extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jchar,
    pub CallCharMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: va_list,
    ) -> jchar,
    pub CallCharMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jchar,
    pub CallShortMethod:
        extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jshort,
    pub CallShortMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: va_list,
    ) -> jshort,
    pub CallShortMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jshort,
    pub CallIntMethod:
        extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jint,
    pub CallIntMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: va_list,
    ) -> jint,
    pub CallIntMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jint,
    pub CallLongMethod:
        extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jlong,
    pub CallLongMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: va_list,
    ) -> jlong,
    pub CallLongMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jlong,
    pub CallFloatMethod:
        extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jfloat,
    pub CallFloatMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: va_list,
    ) -> jfloat,
    pub CallFloatMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jfloat,
    pub CallDoubleMethod:
        extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jdouble,
    pub CallDoubleMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: va_list,
    ) -> jdouble,
    pub CallDoubleMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jdouble,
    pub CallVoidMethod: extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...),
    pub CallVoidMethodV:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: va_list),
    pub CallVoidMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        methodID: jmethodID,
        args: *const jvalue,
    ),
    pub CallNonvirtualObjectMethod: extern "C" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        ...
    ) -> jobject,
    pub CallNonvirtualObjectMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jobject,
    pub CallNonvirtualObjectMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jobject,
    pub CallNonvirtualBooleanMethod: extern "C" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        ...
    ) -> jboolean,
    pub CallNonvirtualBooleanMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jboolean,
    pub CallNonvirtualBooleanMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jboolean,
    pub CallNonvirtualByteMethod: extern "C" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        ...
    ) -> jbyte,
    pub CallNonvirtualByteMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jbyte,
    pub CallNonvirtualByteMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jbyte,
    pub CallNonvirtualCharMethod: extern "C" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        ...
    ) -> jchar,
    pub CallNonvirtualCharMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jchar,
    pub CallNonvirtualCharMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jchar,
    pub CallNonvirtualShortMethod: extern "C" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        ...
    ) -> jshort,
    pub CallNonvirtualShortMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jshort,
    pub CallNonvirtualShortMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jshort,
    pub CallNonvirtualIntMethod: extern "C" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        ...
    ) -> jint,
    pub CallNonvirtualIntMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jint,
    pub CallNonvirtualIntMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jint,
    pub CallNonvirtualLongMethod: extern "C" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        ...
    ) -> jlong,
    pub CallNonvirtualLongMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jlong,
    pub CallNonvirtualLongMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jlong,
    pub CallNonvirtualFloatMethod: extern "C" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        ...
    ) -> jfloat,
    pub CallNonvirtualFloatMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jfloat,
    pub CallNonvirtualFloatMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jfloat,
    pub CallNonvirtualDoubleMethod: extern "C" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        ...
    ) -> jdouble,
    pub CallNonvirtualDoubleMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jdouble,
    pub CallNonvirtualDoubleMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jdouble,
    pub CallNonvirtualVoidMethod:
        extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, ...),
    pub CallNonvirtualVoidMethodV: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ),
    pub CallNonvirtualVoidMethodA: extern "system" fn(
        env: *mut JNIEnv,
        obj: jobject,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ),
    pub GetFieldID: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        name: *const c_char,
        sig: *const c_char,
    ) -> jfieldID,
    pub GetObjectField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jobject,
    pub GetBooleanField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jboolean,
    pub GetByteField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jbyte,
    pub GetCharField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jchar,
    pub GetShortField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jshort,
    pub GetIntField: extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jint,
    pub GetLongField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jlong,
    pub GetFloatField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jfloat,
    pub GetDoubleField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jdouble,
    pub SetObjectField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jobject),
    pub SetBooleanField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jboolean),
    pub SetByteField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jbyte),
    pub SetCharField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jchar),
    pub SetShortField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jshort),
    pub SetIntField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jint),
    pub SetLongField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jlong),
    pub SetFloatField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jfloat),
    pub SetDoubleField:
        extern "system" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jdouble),
    pub GetStaticMethodID: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        name: *const c_char,
        sig: *const c_char,
    ) -> jmethodID,
    pub CallStaticObjectMethod:
        extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jobject,
    pub CallStaticObjectMethodV: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jobject,
    pub CallStaticObjectMethodA: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jobject,
    pub CallStaticBooleanMethod:
        extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jboolean,
    pub CallStaticBooleanMethodV: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jboolean,
    pub CallStaticBooleanMethodA: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jboolean,
    pub CallStaticByteMethod:
        extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jbyte,
    pub CallStaticByteMethodV: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jbyte,
    pub CallStaticByteMethodA: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jbyte,
    pub CallStaticCharMethod:
        extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jchar,
    pub CallStaticCharMethodV: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jchar,
    pub CallStaticCharMethodA: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jchar,
    pub CallStaticShortMethod:
        extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jshort,
    pub CallStaticShortMethodV: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jshort,
    pub CallStaticShortMethodA: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jshort,
    pub CallStaticIntMethod:
        extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jint,
    pub CallStaticIntMethodV: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jint,
    pub CallStaticIntMethodA: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jint,
    pub CallStaticLongMethod:
        extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jlong,
    pub CallStaticLongMethodV: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jlong,
    pub CallStaticLongMethodA: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jlong,
    pub CallStaticFloatMethod:
        extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jfloat,
    pub CallStaticFloatMethodV: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jfloat,
    pub CallStaticFloatMethodA: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jfloat,
    pub CallStaticDoubleMethod:
        extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jdouble,
    pub CallStaticDoubleMethodV: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: va_list,
    ) -> jdouble,
    pub CallStaticDoubleMethodA: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methodID: jmethodID,
        args: *const jvalue,
    ) -> jdouble,
    pub CallStaticVoidMethod:
        extern "C" fn(env: *mut JNIEnv, cls: jclass, methodID: jmethodID, ...),
    pub CallStaticVoidMethodV:
        extern "system" fn(env: *mut JNIEnv, cls: jclass, methodID: jmethodID, args: va_list),
    pub CallStaticVoidMethodA:
        extern "system" fn(env: *mut JNIEnv, cls: jclass, methodID: jmethodID, args: *const jvalue),
    pub GetStaticFieldID: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        name: *const c_char,
        sig: *const c_char,
    ) -> jfieldID,
    pub GetStaticObjectField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jobject,
    pub GetStaticBooleanField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jboolean,
    pub GetStaticByteField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jbyte,
    pub GetStaticCharField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jchar,
    pub GetStaticShortField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jshort,
    pub GetStaticIntField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jint,
    pub GetStaticLongField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jlong,
    pub GetStaticFloatField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jfloat,
    pub GetStaticDoubleField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jdouble,
    pub SetStaticObjectField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jobject),
    pub SetStaticBooleanField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jboolean),
    pub SetStaticByteField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jbyte),
    pub SetStaticCharField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jchar),
    pub SetStaticShortField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jshort),
    pub SetStaticIntField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jint),
    pub SetStaticLongField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jlong),
    pub SetStaticFloatField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jfloat),
    pub SetStaticDoubleField:
        extern "system" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jdouble),
    pub NewString:
        extern "system" fn(env: *mut JNIEnv, unicode: *const jchar, len: jsize) -> jstring,
    pub GetStringLength: extern "system" fn(env: *mut JNIEnv, str: jstring) -> jsize,
    pub GetStringChars:
        extern "system" fn(env: *mut JNIEnv, str: jstring, isCopy: *mut jboolean) -> *const jchar,
    pub ReleaseStringChars: extern "system" fn(env: *mut JNIEnv, str: jstring, chars: *const jchar),
    pub NewStringUTF: extern "system" fn(env: *mut JNIEnv, utf: *const c_char) -> jstring,
    pub GetStringUTFLength: extern "system" fn(env: *mut JNIEnv, str: jstring) -> jsize,
    pub GetStringUTFChars:
        extern "system" fn(env: *mut JNIEnv, str: jstring, isCopy: *mut jboolean) -> *const c_char,
    pub ReleaseStringUTFChars:
        extern "system" fn(env: *mut JNIEnv, str: jstring, chars: *const c_char),
    pub GetArrayLength: extern "system" fn(env: *mut JNIEnv, array: jarray) -> jsize,
    pub NewObjectArray: extern "system" fn(
        env: *mut JNIEnv,
        len: jsize,
        clazz: jclass,
        init: jobject,
    ) -> jobjectArray,
    pub GetObjectArrayElement:
        extern "system" fn(env: *mut JNIEnv, array: jobjectArray, index: jsize) -> jobject,
    pub SetObjectArrayElement:
        extern "system" fn(env: *mut JNIEnv, array: jobjectArray, index: jsize, val: jobject),
    pub NewBooleanArray: extern "system" fn(env: *mut JNIEnv, len: jsize) -> jbooleanArray,
    pub NewByteArray: extern "system" fn(env: *mut JNIEnv, len: jsize) -> jbyteArray,
    pub NewCharArray: extern "system" fn(env: *mut JNIEnv, len: jsize) -> jcharArray,
    pub NewShortArray: extern "system" fn(env: *mut JNIEnv, len: jsize) -> jshortArray,
    pub NewIntArray: extern "system" fn(env: *mut JNIEnv, len: jsize) -> jintArray,
    pub NewLongArray: extern "system" fn(env: *mut JNIEnv, len: jsize) -> jlongArray,
    pub NewFloatArray: extern "system" fn(env: *mut JNIEnv, len: jsize) -> jfloatArray,
    pub NewDoubleArray: extern "system" fn(env: *mut JNIEnv, len: jsize) -> jdoubleArray,
    pub GetBooleanArrayElements: extern "system" fn(
        env: *mut JNIEnv,
        array: jbooleanArray,
        isCopy: *mut jboolean,
    ) -> *mut jboolean,
    pub GetByteArrayElements: extern "system" fn(
        env: *mut JNIEnv,
        array: jbyteArray,
        isCopy: *mut jboolean,
    ) -> *mut jbyte,
    pub GetCharArrayElements: extern "system" fn(
        env: *mut JNIEnv,
        array: jcharArray,
        isCopy: *mut jboolean,
    ) -> *mut jchar,
    pub GetShortArrayElements: extern "system" fn(
        env: *mut JNIEnv,
        array: jshortArray,
        isCopy: *mut jboolean,
    ) -> *mut jshort,
    pub GetIntArrayElements:
        extern "system" fn(env: *mut JNIEnv, array: jintArray, isCopy: *mut jboolean) -> *mut jint,
    pub GetLongArrayElements: extern "system" fn(
        env: *mut JNIEnv,
        array: jlongArray,
        isCopy: *mut jboolean,
    ) -> *mut jlong,
    pub GetFloatArrayElements: extern "system" fn(
        env: *mut JNIEnv,
        array: jfloatArray,
        isCopy: *mut jboolean,
    ) -> *mut jfloat,
    pub GetDoubleArrayElements: extern "system" fn(
        env: *mut JNIEnv,
        array: jdoubleArray,
        isCopy: *mut jboolean,
    ) -> *mut jdouble,
    pub ReleaseBooleanArrayElements: extern "system" fn(
        env: *mut JNIEnv,
        array: jbooleanArray,
        elems: *mut jboolean,
        mode: jint,
    ),
    pub ReleaseByteArrayElements:
        extern "system" fn(env: *mut JNIEnv, array: jbyteArray, elems: *mut jbyte, mode: jint),
    pub ReleaseCharArrayElements:
        extern "system" fn(env: *mut JNIEnv, array: jcharArray, elems: *mut jchar, mode: jint),
    pub ReleaseShortArrayElements:
        extern "system" fn(env: *mut JNIEnv, array: jshortArray, elems: *mut jshort, mode: jint),
    pub ReleaseIntArrayElements:
        extern "system" fn(env: *mut JNIEnv, array: jintArray, elems: *mut jint, mode: jint),
    pub ReleaseLongArrayElements:
        extern "system" fn(env: *mut JNIEnv, array: jlongArray, elems: *mut jlong, mode: jint),
    pub ReleaseFloatArrayElements:
        extern "system" fn(env: *mut JNIEnv, array: jfloatArray, elems: *mut jfloat, mode: jint),
    pub ReleaseDoubleArrayElements:
        extern "system" fn(env: *mut JNIEnv, array: jdoubleArray, elems: *mut jdouble, mode: jint),
    pub GetBooleanArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jbooleanArray,
        start: jsize,
        l: jsize,
        buf: *mut jboolean,
    ),
    pub GetByteArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jbyteArray,
        start: jsize,
        len: jsize,
        buf: *mut jbyte,
    ),
    pub GetCharArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jcharArray,
        start: jsize,
        len: jsize,
        buf: *mut jchar,
    ),
    pub GetShortArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jshortArray,
        start: jsize,
        len: jsize,
        buf: *mut jshort,
    ),
    pub GetIntArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jintArray,
        start: jsize,
        len: jsize,
        buf: *mut jint,
    ),
    pub GetLongArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jlongArray,
        start: jsize,
        len: jsize,
        buf: *mut jlong,
    ),
    pub GetFloatArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jfloatArray,
        start: jsize,
        len: jsize,
        buf: *mut jfloat,
    ),
    pub GetDoubleArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jdoubleArray,
        start: jsize,
        len: jsize,
        buf: *mut jdouble,
    ),
    pub SetBooleanArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jbooleanArray,
        start: jsize,
        l: jsize,
        buf: *const jboolean,
    ),
    pub SetByteArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jbyteArray,
        start: jsize,
        len: jsize,
        buf: *const jbyte,
    ),
    pub SetCharArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jcharArray,
        start: jsize,
        len: jsize,
        buf: *const jchar,
    ),
    pub SetShortArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jshortArray,
        start: jsize,
        len: jsize,
        buf: *const jshort,
    ),
    pub SetIntArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jintArray,
        start: jsize,
        len: jsize,
        buf: *const jint,
    ),
    pub SetLongArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jlongArray,
        start: jsize,
        len: jsize,
        buf: *const jlong,
    ),
    pub SetFloatArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jfloatArray,
        start: jsize,
        len: jsize,
        buf: *const jfloat,
    ),
    pub SetDoubleArrayRegion: extern "system" fn(
        env: *mut JNIEnv,
        array: jdoubleArray,
        start: jsize,
        len: jsize,
        buf: *const jdouble,
    ),
    pub RegisterNatives: extern "system" fn(
        env: *mut JNIEnv,
        clazz: jclass,
        methods: *const JNINativeMethod,
        nMethods: jint,
    ) -> jint,
    pub UnregisterNatives: extern "system" fn(env: *mut JNIEnv, clazz: jclass) -> jint,
    pub MonitorEnter: extern "system" fn(env: *mut JNIEnv, obj: jobject) -> jint,
    pub MonitorExit: extern "system" fn(env: *mut JNIEnv, obj: jobject) -> jint,
    pub GetJavaVM: extern "system" fn(env: *mut JNIEnv, vm: *mut *mut JavaVM) -> jint,
    pub GetStringRegion: extern "system" fn(
        env: *mut JNIEnv,
        str: jstring,
        start: jsize,
        len: jsize,
        buf: *mut jchar,
    ),
    pub GetStringUTFRegion: extern "system" fn(
        env: *mut JNIEnv,
        str: jstring,
        start: jsize,
        len: jsize,
        buf: *mut c_char,
    ),
    pub GetPrimitiveArrayCritical:
        extern "system" fn(env: *mut JNIEnv, array: jarray, isCopy: *mut jboolean) -> *mut c_void,
    pub ReleasePrimitiveArrayCritical:
        extern "system" fn(env: *mut JNIEnv, array: jarray, carray: *mut c_void, mode: jint),
    pub GetStringCritical: extern "system" fn(
        env: *mut JNIEnv,
        string: jstring,
        isCopy: *mut jboolean,
    ) -> *const jchar,
    pub ReleaseStringCritical:
        extern "system" fn(env: *mut JNIEnv, string: jstring, cstring: *const jchar),
    pub NewWeakGlobalRef: extern "system" fn(env: *mut JNIEnv, obj: jobject) -> jweak,
    pub DeleteWeakGlobalRef: extern "system" fn(env: *mut JNIEnv, ref_: jweak),
    pub ExceptionCheck: extern "system" fn(env: *mut JNIEnv) -> jboolean,
    pub NewDirectByteBuffer:
        extern "system" fn(env: *mut JNIEnv, address: *mut c_void, capacity: jlong) -> jobject,
    pub GetDirectBufferAddress: extern "system" fn(env: *mut JNIEnv, buf: jobject) -> *mut c_void,
    pub GetDirectBufferCapacity: extern "system" fn(env: *mut JNIEnv, buf: jobject) -> jlong,
    pub GetObjectRefType: extern "system" fn(env: *mut JNIEnv, obj: jobject) -> jobjectRefType,
}

// pub type JNIEnv = &'static JNINativeInterface;

pub struct JNIEnv<'a> {
    inner: &'a JNINativeInterface,
}
