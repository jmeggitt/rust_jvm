use crate::jvm::call::NativeManager;
use std::ffi::c_void;

mod java_unsafe;
pub mod reflection;
mod system;

// TODO: mod java_unsafe;

pub fn register_natives(natives: &mut NativeManager) {
    // TODO: may be unnessesary as it is able to find these from the local cdylib symbol table
    natives.register_fn(
        "sun/misc/Unsafe",
        "arrayBaseOffset",
        "(Ljava/lang/Class;)I",
        java_unsafe::Java_sun_misc_Unsafe_arrayBaseOffset as *mut c_void,
    );

    natives.register_fn(
        "sun/misc/Unsafe",
        "arrayIndexScale",
        "(Ljava/lang/Class;)I",
        java_unsafe::Java_sun_misc_Unsafe_arrayIndexScale as *mut c_void,
    );

    natives.register_fn(
        "sun/misc/Unsafe",
        "addressSize",
        "()I",
        java_unsafe::Java_sun_misc_Unsafe_addressSize as *mut c_void,
    );

    natives.register_fn(
        "sun/misc/Unsafe",
        "objectFieldOffset",
        "(Ljava/lang/reflect/Field;)J",
        java_unsafe::Java_sun_misc_Unsafe_objectFieldOffset as *mut c_void,
    );
    natives.register_fn(
        "sun/misc/Unsafe",
        "compareAndSwapObject",
        "(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z",
        java_unsafe::Java_sun_misc_Unsafe_compareAndSwapObject as *mut c_void,
    );
    natives.register_fn(
        "sun/misc/Unsafe",
        "getIntVolatile",
        "(Ljava/lang/Object;J)I",
        java_unsafe::Java_sun_misc_Unsafe_getIntVolatile as *mut c_void,
    );
    natives.register_fn(
        "sun/misc/Unsafe",
        "compareAndSwapInt",
        "(Ljava/lang/Object;JII)Z",
        java_unsafe::Java_sun_misc_Unsafe_compareAndSwapInt as *mut c_void,
    );
    natives.register_fn(
        "sun/misc/Unsafe",
        "allocateMemory",
        "(J)J",
        java_unsafe::Java_sun_misc_Unsafe_allocateMemory as *mut c_void,
    );
    natives.register_fn(
        "sun/misc/Unsafe",
        "freeMemory",
        "(J)V",
        java_unsafe::Java_sun_misc_Unsafe_freeMemory as *mut c_void,
    );
    natives.register_fn(
        "sun/misc/Unsafe",
        "putLong",
        "(JJ)V",
        java_unsafe::Java_sun_misc_Unsafe_putLong__JJ as *mut c_void,
    );
    natives.register_fn(
        "sun/misc/Unsafe",
        "getByte",
        "(J)B",
        java_unsafe::Java_sun_misc_Unsafe_getByte__J as *mut c_void,
    );
}
