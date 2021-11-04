#![allow(dead_code)]
use crate::jvm::mem::{ObjectHandle, ObjectReference};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

#[repr(transparent)]
pub struct ClassHandle(ObjectHandle);

impl ClassHandle {
    #[inline]
    pub fn name(&self) -> String {
        self.unwrap_as_class()
    }

    pub fn is_array(&self) -> bool {
        self.name().starts_with('[')
    }
}

impl From<ObjectHandle> for ClassHandle {
    fn from(obj: ObjectHandle) -> Self {
        assert_eq!(obj.get_class(), "java/lang/Class");
        ClassHandle(obj)
    }
}

impl Deref for ClassHandle {
    type Target = ObjectHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ClassHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
