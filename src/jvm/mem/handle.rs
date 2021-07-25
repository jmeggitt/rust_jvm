use std::any::TypeId;
use std::ops::Deref;

use crate::jvm::mem::{
    ArrayReference, ClassSchema, ConstTypeId, InstanceReference, JavaPrimitive, JavaValue,
    ManualInstanceReference, ObjectReference, ObjectType, RawObject,
};
use gc::{Finalize, Gc, Trace};
use hashbrown::HashMap;
use jni::sys::{
    _jobject, jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort, jvalue,
};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::mem::{transmute, ManuallyDrop};
use std::ptr::NonNull;
use std::sync::Arc;

macro_rules! typed_handle {
    (|$handle:ident -> $out:ident| $action:stmt) => {
        match $handle.memory_layout() {
            ObjectType::Instance => {
                let $out = $handle.expect_instance();
                $action
            }
            ObjectType::Array(jboolean::ID) => {
                let $out = $handle.expect_array::<jboolean>();
                $action
            }
            ObjectType::Array(jbyte::ID) => {
                let $out = $handle.expect_array::<jbyte>();
                $action
            }
            ObjectType::Array(jchar::ID) => {
                let $out = $handle.expect_array::<jchar>();
                $action
            }
            ObjectType::Array(jshort::ID) => {
                let $out = $handle.expect_array::<jshort>();
                $action
            }
            ObjectType::Array(jint::ID) => {
                let $out = $handle.expect_array::<jint>();
                $action
            }
            ObjectType::Array(jlong::ID) => {
                let $out = $handle.expect_array::<jlong>();
                $action
            }
            ObjectType::Array(jfloat::ID) => {
                let $out = $handle.expect_array::<jfloat>();
                $action
            }
            ObjectType::Array(jdouble::ID) => {
                let $out = $handle.expect_array::<jdouble>();
                $action
            }
            ObjectType::Array(<Option<ObjectHandle>>::ID) => {
                let $out = $handle.expect_array::<Option<ObjectHandle>>();
                $action
            }
            _ => {}
        }
    };
}

#[derive(Clone)]
#[repr(transparent)]
pub struct ObjectWrapper<T: 'static + Trace> {
    // ptr: Pin<Rc<T>>,
    ptr: ManuallyDrop<Gc<T>>,
}

impl<T: Trace> ObjectWrapper<T> {
    fn new(val: T) -> Self {
        let ptr = Gc::new(val);

        // Make secondary
        // std::mem::forget(ptr.clone());
        // ObjectWrapper { ptr: Rc::pin(val) }
        ObjectWrapper {
            ptr: ManuallyDrop::new(ptr),
        }
    }

    #[inline]
    pub fn into_raw(self) -> jobject {
        unsafe { Gc::into_raw(ManuallyDrop::into_inner(self.ptr)) as jobject }
        // unsafe { Rc::into_raw(Pin::into_inner_unchecked(self.ptr)) as jobject }
    }

    #[inline]
    pub unsafe fn from_raw_unchecked(ptr: jobject) -> Self {
        let ptr = Gc::from_raw(ptr as _);
        // <Gc<T> as Trace>::root(&ptr);
        ObjectWrapper {
            ptr: ManuallyDrop::new(ptr),
            // ptr: Pin::new_unchecked(Rc::from_raw(ptr as _)),
        }
    }

    #[inline]
    pub unsafe fn from_raw(ptr: jobject) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }

        Some(Self::from_raw_unchecked(ptr))
    }
}

impl<T> ObjectReference for ObjectWrapper<RawObject<T>>
where
    RawObject<T>: Trace,
{
    fn get_class_schema(&self) -> Arc<ClassSchema> {
        self.ptr.schema.clone()
    }
}

impl<T: Trace> Deref for ObjectWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.ptr
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct ObjectHandle(NonNull<_jobject>);

impl Finalize for ObjectHandle {}

unsafe impl Trace for ObjectHandle {
    unsafe fn trace(&self) {
        typed_handle!(|self -> out| out.ptr.trace());
    }

    unsafe fn root(&self) {
        typed_handle!(|self -> out| out.ptr.root());
    }

    unsafe fn unroot(&self) {
        typed_handle!(|self -> out| out.ptr.unroot());
    }

    fn finalize_glue(&self) {
        typed_handle!(|self -> out| out.ptr.finalize_glue());
    }
}

impl ObjectHandle {
    pub fn from_ptr(x: jobject) -> Option<Self> {
        match NonNull::new(x) {
            Some(v) => Some(ObjectHandle(v)),
            None => None,
        }
    }

    pub fn ptr(&self) -> jobject {
        self.0.as_ptr()
    }

    #[inline]
    pub fn unwrap_unknown(self) -> ObjectWrapper<RawObject<()>> {
        let ObjectHandle(ptr) = self;
        unsafe { ObjectWrapper::from_raw(ptr.as_ptr()).unwrap() }
    }

    pub fn expect_instance(&self) -> ObjectWrapper<RawObject<Vec<jvalue>>> {
        if self.memory_layout() != ObjectType::Instance {
            panic!("Expected invalid primitive array");
        }

        unsafe { transmute(self.clone().unwrap_unknown()) }
    }

    pub fn expect_array<T: JavaPrimitive>(&self) -> ObjectWrapper<RawObject<Vec<T>>>
    where
        RawObject<Vec<T>>: Trace,
    {
        if self.memory_layout() != ObjectType::Array(TypeId::of::<T>()) {
            panic!("Expected invalid primitive array");
        }

        unsafe { transmute(self.clone().unwrap_unknown()) }
    }

    /// Get array length for an array of unknown type
    pub fn unknown_array_length(&self) -> Option<usize> {
        Some(match self.memory_layout() {
            ObjectType::Array(jboolean::ID) => self.expect_array::<jboolean>().array_length(),
            ObjectType::Array(jbyte::ID) => self.expect_array::<jbyte>().array_length(),
            ObjectType::Array(jchar::ID) => self.expect_array::<jchar>().array_length(),
            ObjectType::Array(jshort::ID) => self.expect_array::<jshort>().array_length(),
            ObjectType::Array(jint::ID) => self.expect_array::<jint>().array_length(),
            ObjectType::Array(jlong::ID) => self.expect_array::<jlong>().array_length(),
            ObjectType::Array(jfloat::ID) => self.expect_array::<jfloat>().array_length(),
            ObjectType::Array(jdouble::ID) => self.expect_array::<jdouble>().array_length(),
            ObjectType::Array(<Option<ObjectHandle> as ConstTypeId>::ID) => {
                self.expect_array::<Option<ObjectHandle>>().array_length()
            }
            _ => return None,
        })
    }
}

impl Hash for ObjectHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = self.0.as_ptr();
        ptr.hash(state);
    }
}

impl ObjectReference for ObjectHandle {
    fn get_class_schema(&self) -> Arc<ClassSchema> {
        self.unwrap_unknown().get_class_schema()
    }
}

impl Debug for ObjectHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let owned = self.clone();
        match self.memory_layout() {
            ObjectType::Instance => owned.expect_instance().fmt(f),
            ObjectType::Array(jboolean::ID) => owned.expect_array::<jboolean>().fmt(f),
            ObjectType::Array(jbyte::ID) => owned.expect_array::<jbyte>().fmt(f),
            ObjectType::Array(jchar::ID) => owned.expect_array::<jchar>().fmt(f),
            ObjectType::Array(jshort::ID) => owned.expect_array::<jshort>().fmt(f),
            ObjectType::Array(jint::ID) => owned.expect_array::<jint>().fmt(f),
            ObjectType::Array(jlong::ID) => owned.expect_array::<jlong>().fmt(f),
            ObjectType::Array(jfloat::ID) => owned.expect_array::<jfloat>().fmt(f),
            ObjectType::Array(jdouble::ID) => owned.expect_array::<jdouble>().fmt(f),
            ObjectType::Array(<Option<ObjectHandle> as ConstTypeId>::ID) => {
                owned.expect_array::<Option<ObjectHandle>>().fmt(f)
            }
            x => panic!("Unable to hash object of type {:?}", x),
        }
    }
}

impl ObjectHandle {
    /// Allocates a new zeroed object instance
    pub fn new(schema: Arc<ClassSchema>) -> ObjectHandle {
        ObjectHandle(NonNull::new(ObjectWrapper::new(RawObject::new(schema)).into_raw()).unwrap())
    }

    // FIXME: Option<ObjectHandle> may not be restricted to a single pointer!
    pub fn new_array<T: JavaPrimitive + Default>(len: usize) -> ObjectHandle
    where
        RawObject<Vec<T>>: Trace,
    {
        ObjectHandle::array_from_data(vec![T::default(); len])
    }

    pub fn array_from_data<T: JavaPrimitive>(arr: Vec<T>) -> ObjectHandle
    where
        RawObject<Vec<T>>: Trace,
    {
        let raw = RawObject::build_raw(ClassSchema::array_schema::<T>(), arr);
        ObjectHandle(NonNull::new(ObjectWrapper::new(raw).into_raw()).unwrap())
    }

    pub fn from_fields<S: AsRef<str>>(
        schema: Arc<ClassSchema>,
        fields: HashMap<S, JavaValue>,
    ) -> ObjectHandle {
        let raw = RawObject::new(schema);

        for (field, value) in fields {
            let offset = raw.field_offset(field);
            InstanceReference::<jvalue>::write_field(&raw, offset, value.into());
        }

        ObjectHandle(NonNull::new(ObjectWrapper::new(raw).into_raw()).unwrap())
    }
}

impl ObjectHandle {
    pub fn expect_string(&self) -> String {
        // FIXME: This check does not check if a class extends string
        assert_eq!(&self.get_class(), "java/lang/String");

        println!("Unwrapping: {:?}", self);
        let instance = self.expect_instance();
        let data: Option<ObjectHandle> = instance.read_named_field("value");
        let chars = data.unwrap().expect_array::<jchar>();

        println!("Gonna do unsafe stuff");
        unsafe {
            // FIXME: I'm probably messing up the encoding
            let arr = chars.raw_fields();
            // let array: Vec<char> = arr.iter().map(|x| std::char::from_u32(*x as u32)).collect();
            String::from_iter(arr.iter().map(|x| std::char::from_u32(*x as u32).unwrap()))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::jvm::mem::{
        ClassSchema, FieldDescriptor, FieldSchema, ObjectHandle, ObjectReference, ObjectType,
    };
    use crate::jvm::{ClassSchema, FieldSchema, ObjectHandle, ObjectReference, ObjectType};
    use crate::types::FieldDescriptor;
    use gc::{Gc, Trace};
    use hashbrown::HashMap;
    use jni::sys::jint;
    use std::sync::Arc;

    pub fn string_schema() -> Arc<ClassSchema> {
        let mut fields = HashMap::new();
        fields.insert(
            "hash".to_string(),
            FieldSchema {
                offset: 8,
                name: "hash".to_string(),
                desc: FieldDescriptor::Int,
            },
        );
        fields.insert(
            "value".to_string(),
            FieldSchema {
                offset: 0,
                name: "value".to_string(),
                desc: FieldDescriptor::Array(Box::new(FieldDescriptor::Char)),
            },
        );

        Arc::new(ClassSchema {
            name: "java/lang/String".to_string(),
            data_form: ObjectType::Instance,
            super_class: Some(Arc::new(ClassSchema {
                name: "java/lang/Object".to_string(),
                data_form: ObjectType::Instance,
                super_class: None,
                field_offsets: HashMap::new(),
                field_lookup: Vec::new(),
            })),
            field_offsets: fields,
            field_lookup: vec![
                FieldSchema {
                    offset: 0,
                    name: "value".into(),
                    desc: FieldDescriptor::Array(Box::new(FieldDescriptor::Char)),
                },
                FieldSchema {
                    offset: 8,
                    name: "hash".into(),
                    desc: FieldDescriptor::Int,
                },
            ],
        })
    }

    #[test]
    pub fn build_simple() {
        ObjectHandle::new_array::<jint>(234);
    }

    #[test]
    pub fn empty_string() {
        let empty_string = ObjectHandle::new(string_schema());
        assert_eq!(empty_string.memory_layout(), ObjectType::Instance);
        assert_eq!(empty_string.expect_string(), "");
    }

    pub fn check_array() {}
}
