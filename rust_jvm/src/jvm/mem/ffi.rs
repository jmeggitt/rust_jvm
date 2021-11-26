//! Convenience types that can be used in FFI which make it easier to access data when the type is
//! known.

// use crate::jvm::mem::{
//     GcBox, ManualInstanceReference, ObjectHandle, ObjectReference, RawInstanceObject, RawObject,
// };
// use byteorder::{BigEndian, ByteOrder};
// use jni::sys::{jchar, jvalue};
// use std::convert::TryFrom;
// use std::ops::{Deref, DerefMut};

use crate::jvm::call::FlowControl;

#[derive(Debug, Clone)]
pub struct ClassCastError {
    pub received: String,
    pub expected: String,
}

impl From<ClassCastError> for FlowControl {
    fn from(_err: ClassCastError) -> Self {
        // TODO: Fill in fields of ClassCastException
        FlowControl::throw("java/lang/ClassCastException")
    }
}

// #[derive(Copy, Clone)]
// #[repr(transparent)]
// pub struct JavaString(RawInstanceObject);
//
// impl JavaString {
//     pub fn new<S: AsRef<str>>(string: S) -> Self {
//
//     }
// }
//
// impl From<JavaString> for String {
//     fn from(string_obj: JavaString) -> Self {
//         let instance = string_obj.0.lock();
//         let data: ObjectHandle = match instance.read_named_field("value") {
//             Some(v) => v,
//             None => return String::new(),
//         };
//
//         let chars = data.unwrap().expect_array::<jchar>().lock();
//
//         let mut bytes = Vec::with_capacity(chars.len());
//
//         for character in chars {
//             if character <= u8::MAX {
//                 bytes.push(character as u8);
//             } else {
//                 let mut buffer = [0u8; 2];
//                 BigEndian::write_u16(&mut buffer, character);
//                 bytes.extend(&buffer);
//             }
//         }
//
//         match cesu8::from_java_cesu8(&bytes) {
//             Ok(v) => v.to_string(),
//             Err(_) => String::from_utf8_lossy(&bytes),
//         }
//     }
// }
//
//
// impl Deref for JavaString {
//     type Target = RawInstanceObject;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//
// impl DerefMut for JavaString {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }
//
// impl TryFrom<ObjectHandle> for JavaString {
//     type Error = ClassCastError;
//
//     fn try_from(value: ObjectHandle) -> Result<Self, Self::Error> {
//         if value.get_class_schema().direct_instanceof("java/lang/String") {
//             Ok(JavaString(value.expect_instance()))
//         } else {
//             Err(ClassCastError {
//                 received: value.get_class(),
//                 expected: "java/lang/String".into(),
//             })
//         }
//     }
// }
