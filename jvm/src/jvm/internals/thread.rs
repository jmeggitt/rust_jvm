use jni::objects::{JClass, JObject};
use jni::sys::{self, jclass, jobject};
use jni::JNIEnv;

use crate::constant_pool::ClassElement;
use crate::jvm::call::RawJNIEnv;
use crate::jvm::mem::ObjectHandle;
use crate::jvm::JavaEnv;

#[no_mangle]
pub unsafe extern "system" fn Java_java_lang_Thread_currentThread__(
    mut jvm: RawJNIEnv,
    _cls: jclass,
) -> jobject {
    jvm.init_class("java/lang/Thread");
    let id = std::thread::current().id();

    if !jvm.threads.contains_key(&id) {
        let obj = ObjectHandle::new(jvm.class_schema("java/lang/Thread"));
        jvm.threads.insert(id, obj);

        jvm.invoke_virtual(
            ClassElement {
                class: "java/lang/Thread".into(),
                element: "<init>".into(),
                desc: "()V".into(),
            },
            obj,
            vec![],
        )
        .unwrap();

        return obj.ptr();
    }

    jvm.threads.get(&id).unwrap().ptr()
}
