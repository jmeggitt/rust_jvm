use crate::jvm::call::NativeManager;
use jni::objects::JClass;
use jni::JNIEnv;
use std::ffi::c_void;

mod java_unsafe;
pub mod reflection;
mod system;
mod thread;

// TODO: mod java_unsafe;

pub fn register_natives(natives: &mut NativeManager) {
    // natives.register_fn(
    //     "sun/reflect/Reflection",
    //     "getCallerClass",
    //     "()Ljava/lang/Class;",
    //     reflection::Java_sun_reflect_Reflection_getCallerClass__ as *mut c_void,
    // );
    //
    // natives.register_fn(
    //     "sun/reflect/Reflection",
    //     "getCallerClass",
    //     "(I)Ljava/lang/Class;",
    //     reflection::Java_sun_reflect_Reflection_getCallerClass__I as *mut c_void,
    // );
    //
    // natives.register_fn(
    //     "sun/reflect/Reflection",
    //     "getClassAccessFlags",
    //     "(Ljava/lang/Class;)I",
    //     reflection::Java_sun_reflect_Reflection_getClassAccessFlags as *mut c_void,
    // );
    //
    // // Skip default initialization for object so we can overwrite it
    // natives.register_fn(
    //     "java/lang/Object",
    //     "registerNatives",
    //     "()V",
    //     empty as *mut c_void,
    // );

    natives.register_fn(
        "java/lang/Thread",
        "currentThread",
        "()Ljava/lang/Thread",
        thread::Java_java_lang_Thread_currentThread__ as *mut c_void,
    );

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

/// A dummy function to register native functions to avoid/skip their usage later on.
/// This is mainly focused on all of the registerNatives() functions in internal classes.
pub unsafe extern "system" fn empty(_env: *mut JNIEnv, _cls: JClass) {
    warn!("Executed dummy function! No Operation was performed.");
}
