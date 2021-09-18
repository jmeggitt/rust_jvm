use crate::jvm::call::NativeManager;
use std::ffi::c_void;

mod java_unsafe;
pub mod reflection;
mod system;

// TODO: mod java_unsafe;

pub fn register_natives(natives: &mut NativeManager) {
    // TODO: may be unnessesary as it is able to find these from the local cdylib symbol table
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "arrayBaseOffset",
    //     "(Ljava/lang/Class;)I",
    //     java_unsafe::Java_sun_misc_Unsafe_arrayBaseOffset as *mut c_void,
    // );
    //
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "arrayIndexScale",
    //     "(Ljava/lang/Class;)I",
    //     java_unsafe::Java_sun_misc_Unsafe_arrayIndexScale as *mut c_void,
    // );
    //
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "addressSize",
    //     "()I",
    //     java_unsafe::Java_sun_misc_Unsafe_addressSize as *mut c_void,
    // );
    //
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "objectFieldOffset",
    //     "(Ljava/lang/reflect/Field;)J",
    //     java_unsafe::Java_sun_misc_Unsafe_objectFieldOffset as *mut c_void,
    // );
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "compareAndSwapObject",
    //     "(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z",
    //     java_unsafe::Java_sun_misc_Unsafe_compareAndSwapObject as *mut c_void,
    // );
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "getIntVolatile",
    //     "(Ljava/lang/Object;J)I",
    //     java_unsafe::Java_sun_misc_Unsafe_getIntVolatile as *mut c_void,
    // );
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "compareAndSwapInt",
    //     "(Ljava/lang/Object;JII)Z",
    //     java_unsafe::Java_sun_misc_Unsafe_compareAndSwapInt as *mut c_void,
    // );
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "allocateMemory",
    //     "(J)J",
    //     java_unsafe::Java_sun_misc_Unsafe_allocateMemory as *mut c_void,
    // );
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "freeMemory",
    //     "(J)V",
    //     java_unsafe::Java_sun_misc_Unsafe_freeMemory as *mut c_void,
    // );
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "putLong",
    //     "(JJ)V",
    //     java_unsafe::Java_sun_misc_Unsafe_putLong__JJ as *mut c_void,
    // );
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "getByte",
    //     "(J)B",
    //     java_unsafe::Java_sun_misc_Unsafe_getByte__J as *mut c_void,
    // );
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "ensureClassInitialized",
    //     "(Ljava/lang/Class;)V",
    //     java_unsafe::Java_sun_misc_Unsafe_ensureClassInitialized as *mut c_void,
    // );
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "staticFieldOffset",
    //     "(Ljava/lang/reflect/Field;)J",
    //     java_unsafe::Java_sun_misc_Unsafe_staticFieldOffset as *mut c_void,
    // );
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "staticFieldBase",
    //     "(Ljava/lang/reflect/Field;)Ljava/lang/Object;",
    //     java_unsafe::Java_sun_misc_Unsafe_staticFieldBase as *mut c_void,
    // );
    // natives.register_fn(
    //     "sun/misc/Unsafe",
    //     "getObjectVolatile",
    //     "(Ljava/lang/Object;J)Ljava/lang/Object;",
    //     java_unsafe::Java_sun_misc_Unsafe_getObjectVolatile as *mut c_void,
    // );

    use java_unsafe::*;
    let unsafe_functions = [
        // ("registerNatives", "()V", Java_sun_misc_Unsafe_registerNatives as _),
        ("getInt", "(Ljava/lang/Object;J)I", Java_sun_misc_Unsafe_getInt__Ljava_lang_Object_2J as _),
        ("putInt", "(Ljava/lang/Object;JI)V", Java_sun_misc_Unsafe_putInt__Ljava_lang_Object_2JI as _),
        ("getObject", "(Ljava/lang/Object;J)Ljava/lang/Object;", Java_sun_misc_Unsafe_getObject as _),
        ("putObject", "(Ljava/lang/Object;JLjava/lang/Object;)V", Java_sun_misc_Unsafe_putObject as _),
        ("getBoolean", "(Ljava/lang/Object;J)Z", Java_sun_misc_Unsafe_getBoolean as _),
        ("putBoolean", "(Ljava/lang/Object;JZ)V", Java_sun_misc_Unsafe_putBoolean as _),
        ("getByte", "(Ljava/lang/Object;J)B", Java_sun_misc_Unsafe_getByte__Ljava_lang_Object_2J as _),
        ("putByte", "(Ljava/lang/Object;JB)V", Java_sun_misc_Unsafe_putByte__Ljava_lang_Object_2JB as _),
        ("getShort", "(Ljava/lang/Object;J)S", Java_sun_misc_Unsafe_getShort__Ljava_lang_Object_2J as _),
        ("putShort", "(Ljava/lang/Object;JS)V", Java_sun_misc_Unsafe_putShort__Ljava_lang_Object_2JS as _),
        ("getChar", "(Ljava/lang/Object;J)C", Java_sun_misc_Unsafe_getChar__Ljava_lang_Object_2J as _),
        ("putChar", "(Ljava/lang/Object;JC)V", Java_sun_misc_Unsafe_putChar__Ljava_lang_Object_2JC as _),
        ("getLong", "(Ljava/lang/Object;J)J", Java_sun_misc_Unsafe_getLong__Ljava_lang_Object_2J as _),
        ("putLong", "(Ljava/lang/Object;JJ)V", Java_sun_misc_Unsafe_putLong__Ljava_lang_Object_2JJ as _),
        ("getFloat", "(Ljava/lang/Object;J)F", Java_sun_misc_Unsafe_getFloat__Ljava_lang_Object_2J as _),
        ("putFloat", "(Ljava/lang/Object;JF)V", Java_sun_misc_Unsafe_putFloat__Ljava_lang_Object_2JF as _),
        ("getDouble", "(Ljava/lang/Object;J)D", Java_sun_misc_Unsafe_getDouble__Ljava_lang_Object_2J as _),
        ("putDouble", "(Ljava/lang/Object;JD)V", Java_sun_misc_Unsafe_putDouble__Ljava_lang_Object_2JD as _),
        ("getByte", "(J)B", Java_sun_misc_Unsafe_getByte__J as _),
        ("putByte", "(JB)V", Java_sun_misc_Unsafe_putByte__JB as _),
        ("getShort", "(J)S", Java_sun_misc_Unsafe_getShort__J as _),
        ("putShort", "(JS)V", Java_sun_misc_Unsafe_putShort__JS as _),
        ("getChar", "(J)C", Java_sun_misc_Unsafe_getChar__J as _),
        ("putChar", "(JC)V", Java_sun_misc_Unsafe_putChar__JC as _),
        ("getInt", "(J)I", Java_sun_misc_Unsafe_getInt__J as _),
        ("putInt", "(JI)V", Java_sun_misc_Unsafe_putInt__JI as _),
        ("getLong", "(J)J", Java_sun_misc_Unsafe_getLong__J as _),
        ("putLong", "(JJ)V", Java_sun_misc_Unsafe_putLong__JJ as _),
        ("getFloat", "(J)F", Java_sun_misc_Unsafe_getFloat__J as _),
        ("putFloat", "(JF)V", Java_sun_misc_Unsafe_putFloat__JF as _),
        ("getDouble", "(J)D", Java_sun_misc_Unsafe_getDouble__J as _),
        ("putDouble", "(JD)V", Java_sun_misc_Unsafe_putDouble__JD as _),
        ("getAddress", "(J)J", Java_sun_misc_Unsafe_getAddress as _),
        ("putAddress", "(JJ)V", Java_sun_misc_Unsafe_putAddress as _),
        ("allocateMemory", "(J)J", Java_sun_misc_Unsafe_allocateMemory as _),
        ("reallocateMemory", "(JJ)J", Java_sun_misc_Unsafe_reallocateMemory as _),
        ("setMemory", "(Ljava/lang/Object;JJB)V", Java_sun_misc_Unsafe_setMemory as _),
        ("copyMemory", "(Ljava/lang/Object;JLjava/lang/Object;JJ)V", Java_sun_misc_Unsafe_copyMemory as _),
        ("freeMemory", "(J)V", Java_sun_misc_Unsafe_freeMemory as _),
        ("staticFieldOffset", "(Ljava/lang/reflect/Field;)J", Java_sun_misc_Unsafe_staticFieldOffset as _),
        ("objectFieldOffset", "(Ljava/lang/reflect/Field;)J", Java_sun_misc_Unsafe_objectFieldOffset as _),
        ("staticFieldBase", "(Ljava/lang/reflect/Field;)Ljava/lang/Object;", Java_sun_misc_Unsafe_staticFieldBase as _),
        ("shouldBeInitialized", "(Ljava/lang/Class;)Z", Java_sun_misc_Unsafe_shouldBeInitialized as _),
        ("ensureClassInitialized", "(Ljava/lang/Class;)V", Java_sun_misc_Unsafe_ensureClassInitialized as _),
        ("arrayBaseOffset", "(Ljava/lang/Class;)I", Java_sun_misc_Unsafe_arrayBaseOffset as _),
        ("arrayIndexScale", "(Ljava/lang/Class;)I", Java_sun_misc_Unsafe_arrayIndexScale as _),
        ("addressSize", "()I", Java_sun_misc_Unsafe_addressSize as _),
        ("pageSize", "()I", Java_sun_misc_Unsafe_pageSize as _),
        ("defineClass", "(Ljava/lang/String;[BIILjava/lang/ClassLoader;Ljava/security/ProtectionDomain;)Ljava/lang/Class;", Java_sun_misc_Unsafe_defineClass as _),
        ("defineAnonymousClass", "(Ljava/lang/Class;[B[Ljava/lang/Object;)Ljava/lang/Class;", Java_sun_misc_Unsafe_defineAnonymousClass as _),
        ("allocateInstance", "(Ljava/lang/Class;)Ljava/lang/Object;", Java_sun_misc_Unsafe_allocateInstance as _),
        ("monitorEnter", "(Ljava/lang/Object;)V", Java_sun_misc_Unsafe_monitorEnter as _),
        ("monitorExit", "(Ljava/lang/Object;)V", Java_sun_misc_Unsafe_monitorExit as _),
        ("tryMonitorEnter", "(Ljava/lang/Object;)Z", Java_sun_misc_Unsafe_tryMonitorEnter as _),
        ("throwException", "(Ljava/lang/Throwable;)V", Java_sun_misc_Unsafe_throwException as _),
        ("compareAndSwapObject", "(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z", Java_sun_misc_Unsafe_compareAndSwapObject as _),
        ("compareAndSwapInt", "(Ljava/lang/Object;JII)Z", Java_sun_misc_Unsafe_compareAndSwapInt as _),
        ("compareAndSwapLong", "(Ljava/lang/Object;JJJ)Z", Java_sun_misc_Unsafe_compareAndSwapLong as _),
        ("getObjectVolatile", "(Ljava/lang/Object;J)Ljava/lang/Object;", Java_sun_misc_Unsafe_getObjectVolatile as _),
        ("putObjectVolatile", "(Ljava/lang/Object;JLjava/lang/Object;)V", Java_sun_misc_Unsafe_putObjectVolatile as _),
        ("getIntVolatile", "(Ljava/lang/Object;J)I", Java_sun_misc_Unsafe_getIntVolatile as _),
        ("putIntVolatile", "(Ljava/lang/Object;JI)V", Java_sun_misc_Unsafe_putIntVolatile as _),
        ("getBooleanVolatile", "(Ljava/lang/Object;J)Z", Java_sun_misc_Unsafe_getBooleanVolatile as _),
        ("putBooleanVolatile", "(Ljava/lang/Object;JZ)V", Java_sun_misc_Unsafe_putBooleanVolatile as _),
        ("getByteVolatile", "(Ljava/lang/Object;J)B", Java_sun_misc_Unsafe_getByteVolatile as _),
        ("putByteVolatile", "(Ljava/lang/Object;JB)V", Java_sun_misc_Unsafe_putByteVolatile as _),
        ("getShortVolatile", "(Ljava/lang/Object;J)S", Java_sun_misc_Unsafe_getShortVolatile as _),
        ("putShortVolatile", "(Ljava/lang/Object;JS)V", Java_sun_misc_Unsafe_putShortVolatile as _),
        ("getCharVolatile", "(Ljava/lang/Object;J)C", Java_sun_misc_Unsafe_getCharVolatile as _),
        ("putCharVolatile", "(Ljava/lang/Object;JC)V", Java_sun_misc_Unsafe_putCharVolatile as _),
        ("getLongVolatile", "(Ljava/lang/Object;J)J", Java_sun_misc_Unsafe_getLongVolatile as _),
        ("putLongVolatile", "(Ljava/lang/Object;JJ)V", Java_sun_misc_Unsafe_putLongVolatile as _),
        ("getFloatVolatile", "(Ljava/lang/Object;J)F", Java_sun_misc_Unsafe_getFloatVolatile as _),
        ("putFloatVolatile", "(Ljava/lang/Object;JF)V", Java_sun_misc_Unsafe_putFloatVolatile as _),
        ("getDoubleVolatile", "(Ljava/lang/Object;J)D", Java_sun_misc_Unsafe_getDoubleVolatile as _),
        ("putDoubleVolatile", "(Ljava/lang/Object;JD)V", Java_sun_misc_Unsafe_putDoubleVolatile as _),
        ("putOrderedObject", "(Ljava/lang/Object;JLjava/lang/Object;)V", Java_sun_misc_Unsafe_putOrderedObject as _),
        ("putOrderedInt", "(Ljava/lang/Object;JI)V", Java_sun_misc_Unsafe_putOrderedInt as _),
        ("putOrderedLong", "(Ljava/lang/Object;JJ)V", Java_sun_misc_Unsafe_putOrderedLong as _),
        ("unpark", "(Ljava/lang/Object;)V", Java_sun_misc_Unsafe_unpark as _),
        ("park", "(ZJ)V", Java_sun_misc_Unsafe_park as _),
        ("getLoadAverage", "([DI)I", Java_sun_misc_Unsafe_getLoadAverage as _),
        ("loadFence", "()V", Java_sun_misc_Unsafe_loadFence as _),
        ("storeFence", "()V", Java_sun_misc_Unsafe_storeFence as _),
        ("fullFence", "()V", Java_sun_misc_Unsafe_fullFence as _),
    ];

    for (name, sig, fn_ptr) in unsafe_functions {
        natives.register_fn("sun/misc/Unsafe", name, sig, fn_ptr);
    }
}
