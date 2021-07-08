use jni::JNIEnv;
use jni::objects::{JObject, JClass};
use jni::sys::{jint, jdouble, jfloat, jbyte, jlong, jshort, jchar, jboolean};

// TODO: Fill in unsafe

/// Class:     sun_misc_Unsafe
/// Method:    getInt
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getInt__Ljava_lang_Object_2J
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putInt
/// Signature: (Ljava/lang/Object{ unimplemented!() }JI)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putInt__Ljava_lang_Object_2JI
(_env: JNIEnv, _this: JObject, jobject, jlong, jint) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getObject
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)Ljava/lang/Object{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getObject
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putObject
/// Signature: (Ljava/lang/Object{ unimplemented!() }JLjava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putObject
(_env: JNIEnv, _this: JObject, jobject, jlong, jobject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getBoolean
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getBoolean
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putBoolean
/// Signature: (Ljava/lang/Object{ unimplemented!() }JZ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putBoolean
(_env: JNIEnv, _this: JObject, jobject, jlong, jboolean) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getByte
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)B
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getByte__Ljava_lang_Object_2J
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putByte
/// Signature: (Ljava/lang/Object{ unimplemented!() }JB)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putByte__Ljava_lang_Object_2JB
(_env: JNIEnv, _this: JObject, jobject, jlong, jbyte) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getShort
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)S
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getShort__Ljava_lang_Object_2J
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putShort
/// Signature: (Ljava/lang/Object{ unimplemented!() }JS)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putShort__Ljava_lang_Object_2JS
(_env: JNIEnv, _this: JObject, jobject, jlong, jshort) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getChar
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)C
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getChar__Ljava_lang_Object_2J
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putChar
/// Signature: (Ljava/lang/Object{ unimplemented!() }JC)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putChar__Ljava_lang_Object_2JC
(_env: JNIEnv, _this: JObject, jobject, jlong, jchar) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getLong
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getLong__Ljava_lang_Object_2J
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putLong
/// Signature: (Ljava/lang/Object{ unimplemented!() }JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putLong__Ljava_lang_Object_2JJ
(_env: JNIEnv, _this: JObject, jobject, jlong, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getFloat
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)F
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getFloat__Ljava_lang_Object_2J
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putFloat
/// Signature: (Ljava/lang/Object{ unimplemented!() }JF)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putFloat__Ljava_lang_Object_2JF
(_env: JNIEnv, _this: JObject, jobject, jlong, jfloat) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getDouble
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)D
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getDouble__Ljava_lang_Object_2J
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putDouble
/// Signature: (Ljava/lang/Object{ unimplemented!() }JD)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putDouble__Ljava_lang_Object_2JD
(_env: JNIEnv, _this: JObject, jobject, jlong, jdouble) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getByte
/// Signature: (J)B
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getByte__J
(_env: JNIEnv, _this: JObject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putByte
/// Signature: (JB)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putByte__JB
(_env: JNIEnv, _this: JObject, jlong, jbyte) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getShort
/// Signature: (J)S
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getShort__J
(_env: JNIEnv, _this: JObject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putShort
/// Signature: (JS)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putShort__JS
(_env: JNIEnv, _this: JObject, jlong, jshort) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getChar
/// Signature: (J)C
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getChar__J
(_env: JNIEnv, _this: JObject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putChar
/// Signature: (JC)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putChar__JC
(_env: JNIEnv, _this: JObject, jlong, jchar) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getInt
/// Signature: (J)I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getInt__J
(_env: JNIEnv, _this: JObject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putInt
/// Signature: (JI)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putInt__JI
(_env: JNIEnv, _this: JObject, jlong, jint) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getLong
/// Signature: (J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getLong__J
(_env: JNIEnv, _this: JObject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putLong
/// Signature: (JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putLong__JJ
(_env: JNIEnv, _this: JObject, jlong, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getFloat
/// Signature: (J)F
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getFloat__J
(_env: JNIEnv, _this: JObject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putFloat
/// Signature: (JF)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putFloat__JF
(_env: JNIEnv, _this: JObject, jlong, jfloat) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getDouble
/// Signature: (J)D
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getDouble__J
(_env: JNIEnv, _this: JObject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putDouble
/// Signature: (JD)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putDouble__JD
(_env: JNIEnv, _this: JObject, jlong, jdouble) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getAddress
/// Signature: (J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getAddress
(_env: JNIEnv, _this: JObject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putAddress
/// Signature: (JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putAddress
(_env: JNIEnv, _this: JObject, jlong, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    allocateMemory
/// Signature: (J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_allocateMemory
(_env: JNIEnv, _this: JObject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    reallocateMemory
/// Signature: (JJ)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_reallocateMemory
(_env: JNIEnv, _this: JObject, jlong, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    setMemory
/// Signature: (Ljava/lang/Object{ unimplemented!() }JJB)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_setMemory
(_env: JNIEnv, _this: JObject, jobject, jlong, jlong, jbyte) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    copyMemory
/// Signature: (Ljava/lang/Object{ unimplemented!() }JLjava/lang/Object{ unimplemented!() }JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_copyMemory
(_env: JNIEnv, _this: JObject, jobject, jlong, jobject, jlong, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    freeMemory
/// Signature: (J)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_freeMemory
(_env: JNIEnv, _this: JObject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    staticFieldOffset
/// Signature: (Ljava/lang/reflect/Field{ unimplemented!() })J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_staticFieldOffset
(_env: JNIEnv, _this: JObject, jobject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    objectFieldOffset
/// Signature: (Ljava/lang/reflect/Field{ unimplemented!() })J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_objectFieldOffset
(_env: JNIEnv, _this: JObject, jobject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    staticFieldBase
/// Signature: (Ljava/lang/reflect/Field{ unimplemented!() })Ljava/lang/Object{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_staticFieldBase
(_env: JNIEnv, _this: JObject, jobject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    shouldBeInitialized
/// Signature: (Ljava/lang/Class{ unimplemented!() })Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_shouldBeInitialized
(_env: JNIEnv, _this: JObject, jclass) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    ensureClassInitialized
/// Signature: (Ljava/lang/Class{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_ensureClassInitialized
(_env: JNIEnv, _this: JObject, jclass) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    arrayBaseOffset
/// Signature: (Ljava/lang/Class{ unimplemented!() })I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_arrayBaseOffset
(_env: JNIEnv, _this: JObject, jclass) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    arrayIndexScale
/// Signature: (Ljava/lang/Class{ unimplemented!() })I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_arrayIndexScale
(_env: JNIEnv, _this: JObject, jclass) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    addressSize
/// Signature: ()I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_addressSize
(_env: JNIEnv, _this: JObject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    pageSize
/// Signature: ()I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_pageSize
(_env: JNIEnv, _this: JObject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    defineClass
/// Signature: (Ljava/lang/String{ unimplemented!() }[BIILjava/lang/ClassLoader{ unimplemented!() }Ljava/security/ProtectionDomain{ unimplemented!() })Ljava/lang/Class{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_defineClass
(_env: JNIEnv, _this: JObject, jstring, jbyteArray, jint, jint, jobject, jobject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    defineAnonymousClass
/// Signature: (Ljava/lang/Class{ unimplemented!() }[B[Ljava/lang/Object{ unimplemented!() })Ljava/lang/Class{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_defineAnonymousClass
(_env: JNIEnv, _this: JObject, jclass, jbyteArray, jobjectArray) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    allocateInstance
/// Signature: (Ljava/lang/Class{ unimplemented!() })Ljava/lang/Object{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_allocateInstance
(_env: JNIEnv, _this: JObject, jclass) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    monitorEnter
/// Signature: (Ljava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_monitorEnter
(_env: JNIEnv, _this: JObject, jobject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    monitorExit
/// Signature: (Ljava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_monitorExit
(_env: JNIEnv, _this: JObject, jobject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    tryMonitorEnter
/// Signature: (Ljava/lang/Object{ unimplemented!() })Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_tryMonitorEnter
(_env: JNIEnv, _this: JObject, jobject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    throwException
/// Signature: (Ljava/lang/Throwable{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_throwException
(_env: JNIEnv, _this: JObject, jthrowable) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    compareAndSwapObject
/// Signature: (Ljava/lang/Object{ unimplemented!() }JLjava/lang/Object{ unimplemented!() }Ljava/lang/Object{ unimplemented!() })Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapObject
(_env: JNIEnv, _this: JObject, jobject, jlong, jobject, jobject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    compareAndSwapInt
/// Signature: (Ljava/lang/Object{ unimplemented!() }JII)Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapInt
(_env: JNIEnv, _this: JObject, jobject, jlong, jint, jint) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    compareAndSwapLong
/// Signature: (Ljava/lang/Object{ unimplemented!() }JJJ)Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapLong
(_env: JNIEnv, _this: JObject, jobject, jlong, jlong, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getObjectVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)Ljava/lang/Object{ unimplemented!() }
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getObjectVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putObjectVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JLjava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putObjectVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong, jobject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getIntVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getIntVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putIntVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JI)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putIntVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong, jint) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getBooleanVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)Z
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getBooleanVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putBooleanVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JZ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putBooleanVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong, jboolean) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getByteVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)B
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getByteVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putByteVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JB)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putByteVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong, jbyte) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getShortVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)S
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getShortVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putShortVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JS)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putShortVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong, jshort) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getCharVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)C
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getCharVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putCharVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JC)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putCharVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong, jchar) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getLongVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)J
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getLongVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putLongVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putLongVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getFloatVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)F
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getFloatVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putFloatVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JF)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putFloatVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong, jfloat) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getDoubleVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }J)D
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getDoubleVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putDoubleVolatile
/// Signature: (Ljava/lang/Object{ unimplemented!() }JD)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putDoubleVolatile
(_env: JNIEnv, _this: JObject, jobject, jlong, jdouble) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putOrderedObject
/// Signature: (Ljava/lang/Object{ unimplemented!() }JLjava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putOrderedObject
(_env: JNIEnv, _this: JObject, jobject, jlong, jobject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putOrderedInt
/// Signature: (Ljava/lang/Object{ unimplemented!() }JI)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putOrderedInt
(_env: JNIEnv, _this: JObject, jobject, jlong, jint) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    putOrderedLong
/// Signature: (Ljava/lang/Object{ unimplemented!() }JJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putOrderedLong
(_env: JNIEnv, _this: JObject, jobject, jlong, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    unpark
/// Signature: (Ljava/lang/Object{ unimplemented!() })V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_unpark
(_env: JNIEnv, _this: JObject, jobject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    park
/// Signature: (ZJ)V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_park
(_env: JNIEnv, _this: JObject, jboolean, jlong) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    getLoadAverage
/// Signature: ([DI)I
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getLoadAverage
(_env: JNIEnv, _this: JObject, jdoubleArray, jint) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    loadFence
/// Signature: ()V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_loadFence
(_env: JNIEnv, _this: JObject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    storeFence
/// Signature: ()V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_storeFence
(_env: JNIEnv, _this: JObject) { unimplemented!() }

/// Class:     sun_misc_Unsafe
/// Method:    fullFence
/// Signature: ()V
#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_fullFence
(_env: JNIEnv, _this: JObject) { unimplemented!() }