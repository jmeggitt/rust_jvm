use jni::sys::JNIEnv;

use jni::objects::JClass;

use jni::sys::{jclass, jint};
use crate::jvm::JavaEnv;

/*
 * Class:     sun_reflect_Reflection
 * Method:    getCallerClass
 * Signature: ()Ljava/lang/Class;
 */
#[no_mangle]
pub unsafe extern "system" fn Java_sun_reflect_Reflection_getCallerClass__(
    env: *mut JNIEnv,
    _class: JClass,
) -> jclass {
    // let jvm = GLOBAL_JavaEnv.as_mut().unwrap();
    let jvm = &*((&**env).reserved0 as *mut JavaEnv);

    let len = jvm.call_stack.len();

    if len < 3 {
        panic!("Attempted to call Java_sun_reflect_Reflection_getCallerClass__ without caller");
    }

    // len - 1 = Reflection.class
    // len - 2 = Target class
    // len - 3 = Caller class

    let class = jvm.call_stack[len - 3].0.clone();

    // FIXME: Make explicit memory leak because current value is stored on the stack and we can't
    // make a policy of freeing results since it wont apply in all cases. It could be solved by a
    // reference table, but that does not work well with rust.
    // Box::leak(Box::new(class)) as *mut Rc<UnsafeCell<Object>> as jclass
    class.unwrap_unknown().into_raw()
}

// JNIEXPORT jclass JNICALL Java_sun_reflect_Reflection_getCallerClass__
// (JNIEnv *, jclass);

/*
 * Class:     sun_reflect_Reflection
 * Method:    getCallerClass
 * Signature: (I)Ljava/lang/Class;
 */
#[no_mangle]
pub unsafe extern "system" fn Java_sun_reflect_Reflection_getCallerClass__I(
    _env: JNIEnv,
    _class: JClass,
    _x: jint,
) -> jclass {
    unimplemented!()
}
// JNIEXPORT jclass JNICALL Java_sun_reflect_Reflection_getCallerClass__I
// (JNIEnv *, jclass, jint);

/*
 * Class:     sun_reflect_Reflection
 * Method:    getClassAccessFlags
 * Signature: (Ljava/lang/Class;)I
 */
#[no_mangle]
pub unsafe extern "system" fn Java_sun_reflect_Reflection_getClassAccessFlags(
    _env: JNIEnv,
    _class: JClass,
    _target: JClass,
) -> jint {
    unimplemented!()
}
// JNIEXPORT jint JNICALL Java_sun_reflect_Reflection_getClassAccessFlags
// (JNIEnv *, jclass, jclass);
