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
}
