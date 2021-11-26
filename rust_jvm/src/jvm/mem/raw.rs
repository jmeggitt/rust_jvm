use std::ops::{Deref, DerefMut};

use crate::jvm::mem::gc::Trace;
use crate::jvm::mem::{
    ClassSchema, JavaPrimitive, JavaValue, NonCircularDebug, ObjectHandle, ObjectType,
};
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort, jvalue};
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::mem::size_of;
use std::sync::Arc;

#[derive(Clone)]
pub struct RawObject<T: ?Sized> {
    pub schema: Arc<ClassSchema>,
    fields: T,
}

// impl<T: Clone> Clone for RawObject<T> {
//     fn clone(&self) -> Self {
//         RawObject {
//             schema: self.schema.clone(),
//             fields: unsafe { UnsafeCell::new((&*self.fields.get()).clone()) },
//         }
//     }
// }

impl<T> RawObject<T> {
    pub fn build_raw(schema: Arc<ClassSchema>, fields: T) -> Self {
        RawObject { schema, fields }
    }

    // /// # Safety
    // /// For correct usage, the type of the object might be check before attempting to read the raw
    // /// fields. If this is not done, it will give incorrect values.
    // #[allow(clippy::mut_from_ref)]
    // pub unsafe fn raw_fields(&self) -> &mut T {
    //     &mut *self.fields.get()
    // }
}

impl<T> RawObject<Vec<T>> {
    pub fn base_ptr(&self) -> *mut () {
        self.fields.as_ptr() as *mut ()
        // unsafe {
        //     let fields = &*self.fields.get();
        //     fields.as_ptr() as *mut ()
        // }
    }
}

pub struct GcIter<T: ?Sized> {
    index: usize,
    inner: T,
}

impl<'a> Iterator for GcIter<&'a RawObject<Vec<jvalue>>> {
    type Item = ObjectHandle;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.inner.schema.field_lookup.len() {
                return None;
            }

            if self.inner.schema.field_lookup[self.index].desc.is_object() {
                let ret: Option<ObjectHandle> = self
                    .inner
                    .read_field(self.inner.schema.field_lookup[self.index].offset);
                if let Some(v) = ret {
                    self.index += 1;
                    return Some(v);
                }
            }

            self.index += 1;
        }
    }
}

impl<'a> Iterator for GcIter<&'a RawObject<Vec<Option<ObjectHandle>>>> {
    type Item = ObjectHandle;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.inner.array_length() {
                return None;
            }

            let obj = self.inner.read_array(self.index);
            self.index += 1;
            if let Some(v) = obj {
                return Some(v);
            }
        }
    }
}

pub trait IntoGcIter {
    fn gc_iter(&self) -> GcIter<&Self>;
}

impl IntoGcIter for RawObject<Vec<jvalue>> {
    fn gc_iter(&self) -> GcIter<&Self> {
        GcIter {
            inner: self,
            index: 0,
        }
    }
}

impl IntoGcIter for RawObject<Vec<Option<ObjectHandle>>> {
    fn gc_iter(&self) -> GcIter<&Self> {
        GcIter {
            inner: self,
            index: 0,
        }
    }
}

// impl<T> Finalize for RawObject<T> {}

unsafe impl<T> Trace for RawObject<T>
where
    Self: IntoGcIter,
    for<'a> GcIter<&'a Self>: Iterator<Item = ObjectHandle>,
{
    unsafe fn trace(&self) {
        for obj in self.gc_iter() {
            obj.trace();
        }
    }
}

macro_rules! empty_trace {
    ($type:ty) => {
        unsafe impl Trace for $type {
            unsafe fn trace(&self) {}
        }
    };
}

// These types have no further sub-objects, but I still need to implement Trace for them
empty_trace!(RawObject<Vec<jboolean>>);
empty_trace!(RawObject<Vec<jbyte>>);
empty_trace!(RawObject<Vec<jchar>>);
empty_trace!(RawObject<Vec<jshort>>);
empty_trace!(RawObject<Vec<jint>>);
empty_trace!(RawObject<Vec<jlong>>);
empty_trace!(RawObject<Vec<jfloat>>);
empty_trace!(RawObject<Vec<jdouble>>);
empty_trace!(RawObject<()>);

impl NonCircularDebug for RawObject<Vec<jvalue>> {
    fn non_cyclical_fmt(
        &self,
        f: &mut Formatter<'_>,
        touched: &mut HashSet<ObjectHandle>,
    ) -> std::fmt::Result {
        if self.get_class() == "java/lang/String" {
            let data: Option<ObjectHandle> = self.read_named_field("value");
            if let Some(arr) = data.map(|x| x.expect_array::<jchar>()) {
                let lock = arr.lock();
                let len = lock.array_length();
                let mut out = String::new();
                for idx in 0..len {
                    out.push(std::char::from_u32(lock.read_array(idx) as u32).unwrap());
                }
                write!(f, "{:?}", out)
            } else {
                write!(f, "String Error")
            }
        } else {
            write!(f, "{} {{ ", &self.schema.name)?;

            for field in &self.schema.field_lookup {
                let value: JavaValue = self.read_field(field.offset);
                write!(f, "{}: ", &field.name)?;
                value.non_cyclical_fmt(f, touched)?;
                write!(f, ", ")?;
            }

            write!(f, "}}")
        }
    }
}

impl Debug for RawObject<Vec<jvalue>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut touched = HashSet::new();
        self.non_cyclical_fmt(f, &mut touched)
    }
}

impl<T: JavaPrimitive + NonCircularDebug> NonCircularDebug for RawObject<Vec<T>>
where
    Self: ArrayReference<T>,
{
    fn non_cyclical_fmt(
        &self,
        f: &mut Formatter<'_>,
        touched: &mut HashSet<ObjectHandle>,
    ) -> std::fmt::Result {
        match T::ID {
            jboolean::ID => write!(f, "boolean"),
            jbyte::ID => write!(f, "byte"),
            jchar::ID => write!(f, "char"),
            jshort::ID => write!(f, "short"),
            jint::ID => write!(f, "int"),
            jlong::ID => write!(f, "long"),
            jfloat::ID => write!(f, "float"),
            jdouble::ID => write!(f, "double"),
            ObjectHandle::ID => write!(f, "Object"),
            <Option<ObjectHandle>>::ID => write!(f, "Object"),
        }?;

        write!(f, "[")?;
        self.fields.non_cyclical_fmt(f, touched)?;
        // write!(f, "[{:?}]", &*self.fields.get())
        write!(f, "]")
    }
}

impl<T: JavaPrimitive + NonCircularDebug> Debug for RawObject<Vec<T>>
where
    Self: ArrayReference<T>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut touched = HashSet::new();
        self.non_cyclical_fmt(f, &mut touched)
        // match TypeId::of::<T>() {
        //     jboolean::ID => write!(f, "boolean"),
        //     jbyte::ID => write!(f, "byte"),
        //     jchar::ID => write!(f, "char"),
        //     jshort::ID => write!(f, "short"),
        //     jint::ID => write!(f, "int"),
        //     jlong::ID => write!(f, "long"),
        //     jfloat::ID => write!(f, "float"),
        //     jdouble::ID => write!(f, "double"),
        //     ObjectHandle::ID => write!(f, "Object"),
        //     _ => write!(f, "{}", type_name::<T>()),
        // }?;
        //
        // unsafe { write!(f, "[{:?}]", &*self.fields.get()) }
    }
}

impl RawObject<Vec<jvalue>> {
    pub fn new(schema: Arc<ClassSchema>) -> Self {
        assert_eq!(schema.data_form, ObjectType::Instance);
        RawObject {
            fields: vec![jvalue { j: 0 }; schema.field_offsets.len()],
            // fields: UnsafeCell::new(vec![jvalue { j: 0 }; schema.field_offsets.len()]),
            schema,
        }
    }

    pub fn field_offset<S: AsRef<str>>(&self, field: S) -> usize {
        self.schema.field_offset(field)
    }
}

impl Hash for RawObject<Vec<jvalue>> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        assert!(self.memory_layout().is_instance());

        for field in &self.schema.field_lookup {
            let local: JavaValue = self.read_field(field.offset);
            local.hash(state);
        }
    }
}

impl<T: JavaPrimitive + Hash> Hash for RawObject<Vec<T>>
where
    Self: ArrayReference<T>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.fields.hash(state);
        // unsafe {
        //     let fields = &*self.fields.get();
        //     fields.hash(state);
        // }
    }
}

pub trait ObjectReference {
    fn get_class_schema(&self) -> Arc<ClassSchema>;

    fn memory_layout(&self) -> ObjectType {
        self.get_class_schema().data_form
    }

    fn get_class(&self) -> String {
        self.get_class_schema().name.to_owned()
    }
}

impl<T: ?Sized> ObjectReference for RawObject<T> {
    fn get_class_schema(&self) -> Arc<ClassSchema> {
        self.schema.clone()
    }
}

pub trait InstanceReference<T>: ObjectReference {
    fn write_field(&mut self, offset: usize, val: T);
    fn read_field(&self, offset: usize) -> T;
}

impl InstanceReference<jvalue> for RawObject<Vec<jvalue>> {
    fn write_field(&mut self, offset: usize, val: jvalue) {
        assert_eq!(offset % size_of::<jvalue>(), 0);
        self.fields[offset / size_of::<jvalue>()] = val;
        // unsafe {
        //     let fields = &mut *self.fields.get();
        //     assert!(index < fields.len());
        //     fields[index] = val;
        // }
    }

    fn read_field(&self, offset: usize) -> jvalue {
        assert_eq!(offset % size_of::<jvalue>(), 0);
        self.fields[offset / size_of::<jvalue>()]

        // unsafe {
        //     let fields = &*self.fields.get();
        //     assert!(index < fields.len());
        //     fields[index]
        // }
    }
}

impl InstanceReference<JavaValue> for RawObject<Vec<jvalue>> {
    fn write_field(&mut self, offset: usize, val: JavaValue) {
        let field = self.schema.get_field_from_offset(offset);
        if let Some(v) = field.desc.assign_from(val) {
            <Self as InstanceReference<jvalue>>::write_field(self, offset, v.into());
        } else {
            panic!("{:?} does not match {:?}", field.desc, val);
        }
    }

    fn read_field(&self, offset: usize) -> JavaValue {
        let field = self.schema.get_field_from_offset(offset);
        field
            .desc
            .cast(self.read_field(offset))
            .expect("field can not be cast to local")
    }
}

impl<T: JavaPrimitive> InstanceReference<T> for RawObject<Vec<jvalue>> {
    fn write_field(&mut self, offset: usize, val: T) {
        self.write_field(offset, val.pack())
    }

    fn read_field(&self, offset: usize) -> T {
        T::unpack(self.read_field(offset))
    }
}

/// Convenience trait to manually reading and writing fields by name without first getting the
/// offsets.
pub trait ManualInstanceReference<T>: InstanceReference<T> {
    fn write_named_field<S: AsRef<str>>(&mut self, field: S, val: T);
    fn read_named_field<S: AsRef<str>>(&self, field: S) -> T;
}

impl<P, T: InstanceReference<P>> ManualInstanceReference<P> for T {
    fn write_named_field<S: AsRef<str>>(&mut self, field: S, val: P) {
        let offset = self.get_class_schema().field_offset(field);
        self.write_field(offset, val);
    }

    fn read_named_field<S: AsRef<str>>(&self, field: S) -> P {
        let offset = self.get_class_schema().field_offset(field);
        self.read_field(offset)
    }
}

pub trait ArrayReference<T: JavaPrimitive>: ObjectReference {
    fn write_array(&mut self, index: usize, val: T);
    fn read_array(&self, index: usize) -> T;
    fn array_length(&self) -> usize;
}

impl<T: JavaPrimitive> ArrayReference<T> for RawObject<Vec<T>> {
    fn write_array(&mut self, index: usize, val: T) {
        self.fields[index] = val;

        // unsafe {
        //     let array = &mut *self.fields.get();
        //     assert!(index < array.len());
        //     array[index] = val;
        // }
    }

    fn read_array(&self, index: usize) -> T {
        self.fields[index]
        // unsafe {
        //     let array = &*self.fields.get();
        //     assert!(index < array.len());
        //     array[index]
        // }
    }

    fn array_length(&self) -> usize {
        self.fields.len()
        // unsafe {
        //     let array = &*self.fields.get();
        //     array.len()
        // }
    }
}

impl<T: JavaPrimitive> RawObject<Vec<T>>
where
    Self: Trace,
{
    pub fn array_copy(&self, dst: ObjectHandle, src_pos: usize, dst_pos: usize, len: usize) {
        let dst_array = dst.expect_array::<T>();
        let mut dst_lock = dst_array.lock();
        // let src_vec = &*self.fields.get();
        // let dst_vec = &mut *dst_array.deref().fields.get();
        dst_lock.fields[dst_pos..dst_pos + len]
            .copy_from_slice(&self.fields[src_pos..src_pos + len]);

        // unsafe {
        //     let src_vec = &*self.fields.get();
        //     let dst_vec = &mut *dst_array.deref().fields.get();
        //     dst_vec[dst_pos..dst_pos + len].copy_from_slice(&src_vec[src_pos..src_pos + len]);
        // }
    }
}

impl<T> Deref for RawObject<Vec<T>> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.fields
        // unsafe { &*self.fields.get() }
    }
}

impl<T> DerefMut for RawObject<Vec<T>> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.fields
        // unsafe { &mut *self.fields.get() }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum JavaTypeEnum {
    Boolean,
    Byte,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Reference,
    NonNullReference,
}

/// Utility trait to match type ids
pub trait ConstTypeId {
    const ID: JavaTypeEnum;
}

impl ConstTypeId for jboolean {
    const ID: JavaTypeEnum = JavaTypeEnum::Boolean;
}

impl ConstTypeId for jbyte {
    const ID: JavaTypeEnum = JavaTypeEnum::Byte;
}

impl ConstTypeId for jshort {
    const ID: JavaTypeEnum = JavaTypeEnum::Short;
}

impl ConstTypeId for jchar {
    const ID: JavaTypeEnum = JavaTypeEnum::Char;
}

impl ConstTypeId for jint {
    const ID: JavaTypeEnum = JavaTypeEnum::Int;
}

impl ConstTypeId for jlong {
    const ID: JavaTypeEnum = JavaTypeEnum::Long;
}

impl ConstTypeId for jfloat {
    const ID: JavaTypeEnum = JavaTypeEnum::Float;
}

impl ConstTypeId for jdouble {
    const ID: JavaTypeEnum = JavaTypeEnum::Double;
}

impl ConstTypeId for Option<ObjectHandle> {
    const ID: JavaTypeEnum = JavaTypeEnum::Reference;
}

/// I doubt this will ever apply, but use it for matching debug formatting
impl ConstTypeId for ObjectHandle {
    const ID: JavaTypeEnum = JavaTypeEnum::NonNullReference;
}
