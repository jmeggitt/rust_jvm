use jni::objects::{JClass, JObject};
use jni::sys::{self, jclass, jobject};
use jni::JNIEnv;

use crate::constant_pool::ClassElement;
use crate::jvm::call::{JavaEnvInvoke, RawJNIEnv};
use crate::jvm::mem::{InstanceReference, JavaValue, ManualInstanceReference, ObjectHandle};
use crate::jvm::JavaEnv;

#[no_mangle]
pub unsafe extern "system" fn Java_java_lang_Thread_currentThread__(
    mut env: RawJNIEnv,
    _cls: jclass,
) -> jobject {
    let id = std::thread::current().id();
    // let mut jvm = env.write();

    if !env.read().threads.contains_key(&id) {
        env.init_class("java/lang/Thread");

        // Check if the system thread group has been initialized

        let group = if env.read().sys_thread_group.is_some() {
            env.read().sys_thread_group.unwrap()
        } else {
            let sys_group = ObjectHandle::new(env.write().class_schema("java/lang/ThreadGroup"));

            env.invoke_virtual(
                ClassElement {
                    class: "java/lang/ThreadGroup".into(),
                    element: "<init>".into(),
                    desc: "()V".into(),
                },
                sys_group,
                vec![],
            )
            .unwrap();

            sys_group
        };

        let obj = {
            let mut jvm = env.write();
            let obj = ObjectHandle::new(jvm.class_schema("java/lang/Thread"));
            jvm.threads.insert(id, obj);
            obj
        };

        // Thread must be set up manually :(
        // TODO: Not setting contextClassLoader may cause issues later on
        let thread_id = env
            .invoke_static(
                ClassElement::new("java/lang/Thread", "nextThreadID", "()J"),
                vec![],
            )
            .unwrap()
            .unwrap();
        let instance = obj.expect_instance();
        instance.write_named_field("tid", thread_id.clone());
        instance.write_named_field("priority", JavaValue::Int(5));

        if let JavaValue::Long(tid) = thread_id {
            if tid == 0 {
                instance.write_named_field("name", env.write().build_string("main"));
            } else {
                instance.write_named_field(
                    "name",
                    env.write().build_string(&format!("Thread-{}", tid)),
                );
            }
        } else {
            panic!("Invalid thread ID!");
        }

        // jvm.invoke_virtual(
        //     ClassElement {
        //         class: "java/lang/Thread".into(),
        //         element: "<init>".into(),
        //         desc: "()V".into(),
        //     },
        //     obj,
        //     vec![],
        // )
        //     .unwrap();

        env.invoke_virtual(
            ClassElement {
                class: "java/lang/ThreadGroup".into(),
                element: "add".into(),
                desc: "(Ljava/lang/Thread;)V".into(),
            },
            group,
            vec![JavaValue::Reference(Some(obj))],
        )
        .unwrap();

        obj.expect_instance()
            .write_named_field("group", Some(group));

        if env.read().sys_thread_group.is_none() {
            env.write().sys_thread_group = Some(group);
        }

        return obj.ptr();
    }

    env.read().threads.get(&id).unwrap().ptr()
}
