use jni::objects::{JClass, JObject};
use jni::sys::{self, jclass, jobject};
use jni::JNIEnv;

// use jvm::constant_pool::ClassElement;
// use jvm::jvm::call::RawJNIEnv;
// use jvm::jvm::mem::ObjectHandle;
// use jvm::jvm::JavaEnv;

// #[no_mangle]
// pub unsafe extern "system" fn Java_java_security_AccessController_doPrivileged__Ljava_security_PrivilegedAction_2(
//     env: JNIEnv,
//     cls: JClass,
//     obj: JObject,
// ) -> jobject {
//     match env
//         .call_method(obj, "run", "()Ljava/lang/Object;", &[])
//         .unwrap()
//         .l()
//     {
//         Ok(v) => v.into_inner(),
//         Err(e) => panic!("{:?}", e),
//     }
// }

// #[no_mangle]
// pub unsafe extern "system" fn Java_java_lang_Thread_currentThread__(
//     mut jvm: RawJNIEnv,
//     _cls: jclass,
// ) -> jobject {
//     println!("A");
//     // let jvm = &mut *((&**env).reserved0 as *mut JavaEnv);
//     jvm.init_class("java/lang/Thread");
//     let id = std::thread::current().id();
//
//     if !jvm.threads.contains_key(&id) {
//         let obj = ObjectHandle::new(jvm.class_schema("java/lang/Thread"));
//         jvm.threads.insert(id, obj);
//
//         jvm.invoke_virtual(
//             ClassElement {
//                 class: "java/lang/Thread".into(),
//                 element: "<init>".into(),
//                 desc: "()V".into(),
//             },
//             obj,
//             vec![],
//         );
//
//         return obj.ptr();
//     }
//
//     jvm.threads.get(&id).unwrap().ptr()
// }
