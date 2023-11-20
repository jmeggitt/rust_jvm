use crate::jvm::mem::gc::{GcBox, Trace};
use crate::jvm::mem::string::JavaString;
use crate::jvm::mem::{
    ArrayReference, ClassSchema, ConstTypeId, InstanceReference, JavaPrimitive, JavaValue,
    ManualInstanceReference, NonCircularDebug, ObjectReference, ObjectType, RawObject,
};
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort, jvalue};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::mem::transmute;
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
            _ => panic!(),
        }
    };
}

// /// The ObjectWrapper struct is responsible for acting as a box to hold the raw object. This layer
// /// is also responsible for managing garbage collection primitives. (But at the moment gc is kinda
// /// broken)
// #[derive(Clone)]
// #[repr(transparent)]
// pub struct ObjectWrapper<T: 'static + Trace> {
//     ptr: ManuallyDrop<Gc<T>>,
// }
//
// impl<T: Trace> ObjectWrapper<T> {
//     pub fn new(val: T) -> Self {
//         let ptr = Gc::new(val);
//
//         // Make secondary
//         // std::mem::forget(ptr.clone());
//         // ObjectWrapper { ptr: Rc::pin(val) }
//         ObjectWrapper {
//             ptr: ManuallyDrop::new(ptr),
//         }
//     }
//
//     #[inline]
//     pub fn into_raw(self) -> jobject {
//         Gc::into_raw(ManuallyDrop::into_inner(self.ptr)) as jobject
//         // unsafe { Rc::into_raw(Pin::into_inner_unchecked(self.ptr)) as jobject }
//     }
//
//     /// # Safety
//     /// Not sure why I added this one, but the given pointer must be non-null and point to a
//     /// Gc<RawObject<?>>.
//     #[inline]
//     pub unsafe fn from_raw_unchecked(ptr: jobject) -> Self {
//         let ptr = Gc::from_raw(ptr as _);
//         // <Gc<T> as Trace>::root(&ptr);
//         ObjectWrapper {
//             ptr: ManuallyDrop::new(ptr),
//         }
//     }
//
//     #[inline]
//     pub fn from_raw(ptr: jobject) -> Option<Self> {
//         if ptr.is_null() {
//             return None;
//         }
//
//         unsafe { Some(Self::from_raw_unchecked(ptr)) }
//     }
// }

pub type RawInstanceObject = GcBox<RawObject<Vec<jvalue>>>;
pub type RawArrayObject<T> = GcBox<RawObject<Vec<T>>>;

impl<T> ObjectReference for GcBox<RawObject<T>>
where
    RawObject<T>: Trace,
{
    fn get_class_schema(&self) -> Arc<ClassSchema> {
        let lock = self.lock();
        lock.schema.clone()
        // self.ptr.schema.clone()
    }
}

// impl<T: Trace> Deref for ObjectWrapper<T> {
//     type Target = T;
//
//     fn deref(&self) -> &Self::Target {
//         &*self.ptr
//     }
// }

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct ObjectHandle(GcBox<RawObject<()>>);
// pub type ObjectHandle = GcBox<RawObject<()>>;

impl<T> GcBox<RawObject<T>> {
    pub fn into_unknown(self) -> ObjectHandle {
        unsafe { transmute(self) }
    }
}

impl<T> From<GcBox<RawObject<T>>> for ObjectHandle {
    fn from(x: GcBox<RawObject<T>>) -> Self {
        x.into_unknown()
    }
}

impl PartialEq for ObjectHandle {
    fn eq(&self, other: &Self) -> bool {
        // TODO: Remove this extra logic as it is no-longer needed
        // Enforce string comparison to get around annoying string internment issues
        let class = self.get_class();
        if class == other.get_class() && class == "java/lang/String" {
            return self.expect_string() == other.expect_string();
        }

        // self.ptr() == other.ptr()
        self.0 == other.0
    }
}

impl Eq for ObjectHandle {}

// ObjectHandle needs to pretend to be thread safe to mimic the functionality of Java
// unsafe impl Sync for ObjectHandle {}
//
// unsafe impl Send for ObjectHandle {}

// impl Finalize for ObjectHandle {}

unsafe impl Trace for ObjectHandle {
    unsafe fn trace(&self) {
        typed_handle!(|self -> out| {out.lock().trace();});
    }
}

impl ObjectHandle {
    pub fn from_ptr(x: jobject) -> Option<Self> {
        GcBox::from_ptr(x).map(ObjectHandle)
        // NonNull::new(x).map(|x| ObjectHandle(x as _))
    }

    pub fn ptr(&self) -> jobject {
        self.0.as_ptr()
    }

    // #[inline]
    // pub fn unwrap_unknown(self) -> ObjectWrapper<RawObject<()>> {
    //     let ObjectHandle(ptr) = self;
    //     ObjectWrapper::from_raw(ptr.as_ptr()).unwrap()
    // }

    #[inline]
    pub fn as_instance(&self) -> Option<RawInstanceObject> {
        match self.memory_layout() {
            ObjectType::Instance => unsafe { Some(transmute(self.0)) },
            _ => None,
        }
    }

    #[inline]
    pub fn as_array<T: JavaPrimitive>(&self) -> Option<RawArrayObject<T>>
    where
        RawObject<Vec<T>>: Trace,
    {
        if self.memory_layout() == ObjectType::Array(T::ID) {
            return unsafe { Some(transmute(self.0)) };
        }

        None
    }

    pub fn expect_instance(&self) -> RawInstanceObject {
        match self.as_instance() {
            Some(v) => v,
            None => panic!("Expected instance, got {:?}", self),
        }
    }

    pub fn expect_array<T: JavaPrimitive>(&self) -> RawArrayObject<T>
    where
        RawObject<Vec<T>>: Trace,
    {
        match self.as_array() {
            Some(v) => v,
            None => panic!("Expected instance, got {:?}", self),
        }
    }

    /// Get array length for an array of unknown type
    pub fn unknown_array_length(&self) -> Option<usize> {
        Some(match self.memory_layout() {
            ObjectType::Array(jboolean::ID) => {
                self.expect_array::<jboolean>().lock().array_length()
            }
            ObjectType::Array(jbyte::ID) => self.expect_array::<jbyte>().lock().array_length(),
            ObjectType::Array(jchar::ID) => self.expect_array::<jchar>().lock().array_length(),
            ObjectType::Array(jshort::ID) => self.expect_array::<jshort>().lock().array_length(),
            ObjectType::Array(jint::ID) => self.expect_array::<jint>().lock().array_length(),
            ObjectType::Array(jlong::ID) => self.expect_array::<jlong>().lock().array_length(),
            ObjectType::Array(jfloat::ID) => self.expect_array::<jfloat>().lock().array_length(),
            ObjectType::Array(jdouble::ID) => self.expect_array::<jdouble>().lock().array_length(),
            ObjectType::Array(<Option<ObjectHandle> as ConstTypeId>::ID) => self
                .expect_array::<Option<ObjectHandle>>()
                .lock()
                .array_length(),
            _ => return None,
        })
    }

    /// TODO: Remove
    /// # Safety
    /// This is only intended for use supporting sun/misc/Unsafe. Even then, I am hesitant to add
    /// this function as it is highly likely to result in a segfault if used to get a value outside
    /// an object.
    pub unsafe fn raw_object_memory<T>(&self, offset: usize) -> *mut T {
        // let ObjectHandle(ptr) = self;
        // let raw: ObjectWrapper<RawObject<Vec<jvalue>>> =
        //     ObjectWrapper::from_raw(ptr.as_ptr()).unwrap();
        // let base_ptr: *mut u8 = raw.base_ptr() as *mut u8;
        // base_ptr.offset(offset as isize) as *mut T
        (self.ptr() as *mut u8).add(offset) as *mut T
    }
}

impl Hash for ObjectHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = self.ptr();
        ptr.hash(state);
    }
}

impl ObjectReference for ObjectHandle {
    fn get_class_schema(&self) -> Arc<ClassSchema> {
        self.0.lock().get_class_schema()
    }
}

impl NonCircularDebug for ObjectHandle {
    fn non_cyclical_fmt(
        &self,
        f: &mut Formatter<'_>,
        touched: &mut HashSet<ObjectHandle>,
    ) -> std::fmt::Result {
        // The core of NonCircularDebug is denying object handles that have already been touched
        if touched.contains(self) {
            return write!(f, "@{:p}", self.ptr());
        } else {
            touched.insert(*self);
        }

        match self.memory_layout() {
            ObjectType::Instance => self.expect_instance().lock().non_cyclical_fmt(f, touched),
            ObjectType::Array(jboolean::ID) => self
                .expect_array::<jboolean>()
                .lock()
                .non_cyclical_fmt(f, touched),
            ObjectType::Array(jbyte::ID) => self
                .expect_array::<jbyte>()
                .lock()
                .non_cyclical_fmt(f, touched),
            ObjectType::Array(jchar::ID) => self
                .expect_array::<jchar>()
                .lock()
                .non_cyclical_fmt(f, touched),
            ObjectType::Array(jshort::ID) => self
                .expect_array::<jshort>()
                .lock()
                .non_cyclical_fmt(f, touched),
            ObjectType::Array(jint::ID) => self
                .expect_array::<jint>()
                .lock()
                .non_cyclical_fmt(f, touched),
            ObjectType::Array(jlong::ID) => self
                .expect_array::<jlong>()
                .lock()
                .non_cyclical_fmt(f, touched),
            ObjectType::Array(jfloat::ID) => self
                .expect_array::<jfloat>()
                .lock()
                .non_cyclical_fmt(f, touched),
            ObjectType::Array(jdouble::ID) => self
                .expect_array::<jdouble>()
                .lock()
                .non_cyclical_fmt(f, touched),
            ObjectType::Array(<Option<ObjectHandle> as ConstTypeId>::ID) => self
                .expect_array::<Option<ObjectHandle>>()
                .lock()
                .non_cyclical_fmt(f, touched),
            x => panic!("Unable to hash object of type {:?}", x),
        }
    }
}

impl NonCircularDebug for Option<ObjectHandle> {
    fn non_cyclical_fmt(
        &self,
        f: &mut Formatter<'_>,
        touched: &mut HashSet<ObjectHandle>,
    ) -> std::fmt::Result {
        match self {
            Some(x) => x.non_cyclical_fmt(f, touched),
            None => write!(f, "null"),
        }
    }
}

impl Debug for ObjectHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut touched = HashSet::new();
        self.non_cyclical_fmt(f, &mut touched)
    }
}

impl ObjectHandle {
    /// Allocates a new zeroed object instance
    pub fn new(schema: Arc<ClassSchema>) -> ObjectHandle {
        GcBox::new(RawObject::new(schema)).into_unknown()
        // ObjectHandle(NonNull::new(ObjectWrapper::new(RawObject::new(schema)).into_raw()).unwrap())
    }

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
        GcBox::new(raw).into_unknown()
        // ObjectHandle(NonNull::new(ObjectWrapper::new(raw).into_raw()).unwrap())
    }

    pub fn from_fields<S: AsRef<str>>(
        schema: Arc<ClassSchema>,
        fields: HashMap<S, JavaValue>,
    ) -> ObjectHandle {
        let mut raw = RawObject::new(schema);

        for (field, value) in fields {
            let offset = raw.field_offset(field);
            InstanceReference::<jvalue>::write_field(&mut raw, offset, value.into());
        }

        GcBox::new(raw).into_unknown()
    }
}

impl ObjectHandle {
    pub fn expect_string(&self) -> String {
        JavaString::try_from(*self).unwrap().into()
        // // FIXME: This check does not check if a class extends string
        // assert_eq!(&self.get_class(), "java/lang/String");
        //
        // // println!("Unwrapping: {:?}", self);
        // let instance = self.expect_instance().lock();
        // let data: Option<ObjectHandle> = instance.read_named_field("value");
        // let chars = data.unwrap().expect_array::<jchar>().lock();
        //
        // // println!("Gonna do unsafe stuff");
        // unsafe {
        //     // FIXME: I'm probably messing up the encoding
        //     let arr = chars.raw_fields();
        //     // let array: Vec<char> = arr.iter().map(|x| std::char::from_u32(*x as u32)).collect();
        //     arr.iter()
        //         .map(|x| std::char::from_u32(*x as u32).unwrap())
        //         .collect()
        // }
    }

    pub fn unwrap_as_class(&self) -> String {
        let name: Option<ObjectHandle> = self.expect_instance().lock().read_named_field("name");
        name.unwrap().expect_string().replace('.', "/")
    }
}

#[cfg(test)]
mod test {
    use crate::jvm::mem::{
        ClassSchema, FieldDescriptor, FieldSchema, ObjectHandle, ObjectReference, ObjectType,
    };
    use gc::{Gc, Trace};
    use jni::sys::jint;
    use std::collections::HashMap;
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

    pub fn empty_string() {
        let empty_string = ObjectHandle::new(string_schema());
        assert_eq!(empty_string.memory_layout(), ObjectType::Instance);
        assert_eq!(empty_string.expect_string(), "");
    }
}
