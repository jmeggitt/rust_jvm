use crate::jvm::call::FlowControl;
use crate::jvm::mem::ffi::ClassCastError;
use crate::jvm::mem::{ManualInstanceReference, ObjectHandle, ObjectReference, RawInstanceObject};
use byteorder::{BigEndian, ByteOrder};
use jni::sys::jchar;
use std::convert::TryFrom;
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct JavaString(RawInstanceObject);

impl JavaString {
    pub fn new<S: AsRef<str>>(_string: S) -> Self {
        todo!()
    }
}

impl From<JavaString> for String {
    fn from(string_obj: JavaString) -> Self {
        let instance = string_obj.0.lock();
        let data: ObjectHandle = match instance.read_named_field("value") {
            Some(v) => v,
            None => return String::new(),
        };

        let chars = data.expect_array::<jchar>();
        let char_lock = chars.lock();

        let mut bytes = Vec::with_capacity(char_lock.len());

        for character in char_lock.iter() {
            if *character <= u8::MAX as u16 {
                bytes.push(*character as u8);
            } else {
                let mut buffer = [0u8; 2];
                BigEndian::write_u16(&mut buffer, *character);
                bytes.extend(&buffer);
            }
        }

        match cesu8::from_java_cesu8(&bytes) {
            Ok(v) => v.to_string(),
            Err(_) => String::from_utf8_lossy(&bytes).to_string(),
        }
    }
}

impl Deref for JavaString {
    type Target = RawInstanceObject;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for JavaString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryFrom<ObjectHandle> for JavaString {
    type Error = ClassCastError;

    fn try_from(value: ObjectHandle) -> Result<Self, Self::Error> {
        if value
            .get_class_schema()
            .direct_instanceof("java/lang/String")
        {
            Ok(JavaString(value.expect_instance()))
        } else {
            Err(ClassCastError {
                received: value.get_class(),
                expected: "java/lang/String".into(),
            })
        }
    }
}

// // TODO: Implement these functions here:
// #[no_mangle]
// pub unsafe extern "system" fn JVM_InternString_impl(env: RawJNIEnv, str: jstring) -> jstring {
//     let obj = obj_expect!(env, str, null_mut());
//     let raw_str = obj.expect_string();
//
//     let mut jvm = env.write();
//     if jvm.interned_strings.contains_key(&raw_str) {
//         return jvm.interned_strings.get(&raw_str).unwrap().ptr();
//     }
//
//     jvm.interned_strings.insert(raw_str, obj);
//     str
// }
//
//
// pub unsafe extern "system" fn NewString(
//     env: *mut JNIEnv,
//     unicode: *const jchar,
//     len: jsize,
// ) -> jstring {
//     let env = RawJNIEnv::new(env);
//     let handle = ObjectHandle::new(env.write().class_schema("java/lang/String"));
//     let object = handle.expect_instance();
//
//     let mut chars = vec![0; len as usize];
//     unicode.copy_to(chars.as_mut_ptr(), len as usize);
//
//     object.write_named_field("value", Some(ObjectHandle::array_from_data(chars)));
//     handle.ptr()
// }
//
// pub unsafe extern "system" fn GetStringLength(env: *mut JNIEnv, str: jstring) -> jsize {
//     let env = RawJNIEnv::new(env);
//     obj_expect!(env, str, 0).expect_string().len() as jsize
// }
//
// pub unsafe extern "system" fn GetStringChars(
//     env: *mut JNIEnv,
//     str: jstring,
//     is_copy: *mut jboolean,
// ) -> *const jchar {
//     let env = RawJNIEnv::new(env);
//     if !is_copy.is_null() {
//         *is_copy = JNI_TRUE;
//     }
//
//     let arr: Option<ObjectHandle> = obj_expect!(env, str, null_mut())
//         .expect_instance()
//         .read_named_field("value");
//     let mut clone = arr.unwrap().expect_array::<jchar>().raw_fields().to_vec();
//     let ret = clone.as_mut_ptr();
//     forget(clone); // Forget it so it can be recovered later
//     ret
// }
//
//
// pub unsafe extern "system" fn ReleaseStringChars(
//     env: *mut JNIEnv,
//     str: jstring,
//     chars: *const jchar,
// ) {
//     let env = RawJNIEnv::new(env);
//     let obj: Option<ObjectHandle> = obj_expect!(env, str)
//         .expect_instance()
//         .read_named_field("value");
//     let arr = obj.unwrap().expect_array::<jchar>();
//
//     // Reclaim elements so they get dropped at the end of the function
//     Vec::from_raw_parts(chars as *mut jchar, arr.len(), arr.len());
// }
//
// pub unsafe extern "system" fn NewStringUTF(env: *mut JNIEnv, utf: *const c_char) -> jstring {
//     if env.is_null() {
//         panic!("Got null for JNIEnv");
//     }
//
//     if utf.is_null() {
//         return null_mut();
//     }
//
//     let input = CStr::from_ptr(utf);
//     let env = RawJNIEnv::new(env);
//     let mut jvm = env.write();
//     jvm.build_string(&input.to_string_lossy())
//         .expect_object()
//         .ptr()
// }
//
// pub unsafe extern "system" fn GetStringUTFLength(env: *mut JNIEnv, str: jstring) -> jsize {
//     let env = RawJNIEnv::new(env);
//     obj_expect!(env, str, 0).expect_string().len() as jsize
// }
//
// pub unsafe extern "system" fn GetStringUTFChars(
//     env: *mut JNIEnv,
//     str: jstring,
//     is_copy: *mut jboolean,
// ) -> *const c_char {
//     let jvm = RawJNIEnv::new(env);
//     if !is_copy.is_null() {
//         *is_copy = JNI_TRUE;
//     }
//
//     let obj = obj_expect!(jvm, str, null());
//     let str = obj.expect_string();
//
//     let ret = CString::new(str).unwrap();
//     ret.into_raw() as _
// }
//
// pub unsafe extern "system" fn ReleaseStringUTFChars(
//     env: *mut JNIEnv,
//     str: jstring,
//     chars: *const c_char,
// ) {
//     // Put the pointer back into CString struct so it will be dropped at the end of the function
//     CString::from_raw(chars as _);
// }
//
