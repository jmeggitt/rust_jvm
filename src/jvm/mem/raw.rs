use std::any::{type_name, TypeId};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::rc::Rc;

use crate::class::{AccessFlags, BufferedRead, Class, FieldInfo};
use crate::jvm::interface::GLOBAL_JVM;
use crate::jvm::JVM;
use crate::jvm::mem::FieldDescriptor;
use gc::{Finalize, Gc, Trace};
use hashbrown::HashMap;
use jni::sys::{
    _jobject, jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort, jvalue,
};
use lazy_static::lazy_static;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::mem::{size_of, transmute, ManuallyDrop};
use std::ptr::{null_mut, NonNull};
use std::sync::Arc;
use crate::jvm::mem::{ClassSchema, ObjectHandle, LocalVariable, JavaPrimitive, ObjectType};


// #[repr(C, align(8))]
pub struct RawObject<T: ?Sized> {
    pub schema: Arc<ClassSchema>,
    fields: UnsafeCell<T>,
}

impl<T> RawObject<T> {
    pub fn build_raw(schema: Arc<ClassSchema>, fields: T) -> Self {
        RawObject {
            schema,
            fields: UnsafeCell::new(fields),
        }
    }

    pub unsafe fn raw_fields(&self) -> &mut T {
        &mut *self.fields.get()
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

impl<T> Finalize for RawObject<T> {}

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

    unsafe fn root(&self) {
        for obj in self.gc_iter() {
            obj.root();
        }
    }

    unsafe fn unroot(&self) {
        for obj in self.gc_iter() {
            obj.unroot();
        }
    }

    fn finalize_glue(&self) {
        for obj in self.gc_iter() {
            obj.finalize_glue();
        }
    }
}

macro_rules! empty_trace {
    ($type:ty) => {
        unsafe impl Trace for $type {
            unsafe fn trace(&self) {}
            unsafe fn root(&self) {}
            unsafe fn unroot(&self) {}
            fn finalize_glue(&self) {}
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

impl Debug for RawObject<Vec<jvalue>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {{ ", &self.schema.name)?;

        for field in &self.schema.field_lookup {
            let value: LocalVariable = self.read_field(field.offset);
            write!(f, "{}: {:?}, ", &field.name, value)?;
        }

        write!(f, "}}")
    }
}

impl<T: JavaPrimitive + Debug> Debug for RawObject<Vec<T>>
    where
        Self: ArrayReference<T>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match TypeId::of::<T>() {
            jboolean::ID => write!(f, "boolean"),
            jbyte::ID => write!(f, "byte"),
            jchar::ID => write!(f, "char"),
            jshort::ID => write!(f, "short"),
            jint::ID => write!(f, "int"),
            jlong::ID => write!(f, "long"),
            jfloat::ID => write!(f, "float"),
            jdouble::ID => write!(f, "double"),
            ObjectHandle::ID => write!(f, "Object"),
            _ => write!(f, "{}", type_name::<T>()),
        }?;

        unsafe { write!(f, "[{:?}]", &*self.fields.get()) }
    }
}



impl RawObject<Vec<jvalue>> {
    pub fn new(schema: Arc<ClassSchema>) -> Self {
        assert_eq!(schema.data_form, ObjectType::Instance);
        RawObject {
            fields: UnsafeCell::new(vec![jvalue { j: 0 }; schema.field_offsets.len()]),
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
            let local: LocalVariable = self.read_field(field.offset);
            local.hash(state);
        }
    }
}

impl<T: JavaPrimitive + Hash> Hash for RawObject<Vec<T>>
    where
        Self: ArrayReference<T>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe {
            let fields = &*self.fields.get();
            fields.hash(state);
        }
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
    fn write_field(&self, offset: usize, val: T);
    fn read_field(&self, offset: usize) -> T;
}

impl InstanceReference<jvalue> for RawObject<Vec<jvalue>> {
    fn write_field(&self, offset: usize, val: jvalue) {
        assert_eq!(offset % size_of::<jvalue>(), 0);
        let index = offset / size_of::<jvalue>();

        unsafe {
            let fields = &mut *self.fields.get();
            assert!(index < fields.len());
            fields[index] = val;
        }
    }

    fn read_field(&self, offset: usize) -> jvalue {
        assert_eq!(offset % size_of::<jvalue>(), 0);
        let index = offset / size_of::<jvalue>();

        unsafe {
            let fields = &*self.fields.get();
            assert!(index < fields.len());
            fields[index]
        }
    }
}

impl InstanceReference<LocalVariable> for RawObject<Vec<jvalue>> {
    fn write_field(&self, offset: usize, val: LocalVariable) {
        let field = self.schema.get_field_from_offset(offset);
        assert!(field.desc.matches(&val));
        <Self as InstanceReference<jvalue>>::write_field(self, offset, val.into());
    }

    fn read_field(&self, offset: usize) -> LocalVariable {
        let field = self.schema.get_field_from_offset(offset);
        field
            .desc
            .cast(self.read_field(offset))
            .expect("field can not be cast to local")
    }
}

impl<T: JavaPrimitive> InstanceReference<T> for RawObject<Vec<jvalue>> {
    fn write_field(&self, offset: usize, val: T) {
        self.write_field(offset, val.pack())
    }

    fn read_field(&self, offset: usize) -> T {
        T::unpack(self.read_field(offset))
    }
}

/// Convenience trait to manually reading and writing fields by name without first getting the
/// offsets.
pub trait ManualInstanceReference<T>: InstanceReference<T> {
    fn write_named_field<S: AsRef<str>>(&self, field: S, val: T);
    fn read_named_field<S: AsRef<str>>(&self, field: S) -> T;
}

impl<P, T: InstanceReference<P>> ManualInstanceReference<P> for T {
    fn write_named_field<S: AsRef<str>>(&self, field: S, val: P) {
        let offset = self.get_class_schema().field_offset(field);
        self.write_field(offset, val);
    }

    fn read_named_field<S: AsRef<str>>(&self, field: S) -> P {
        let offset = self.get_class_schema().field_offset(field);
        self.read_field(offset)
    }
}

pub trait ArrayReference<T: JavaPrimitive>: ObjectReference {
    fn write_array(&self, index: usize, val: T);
    fn read_array(&self, index: usize) -> T;
    fn array_length(&self) -> usize;
}

impl<T: JavaPrimitive> ArrayReference<T> for RawObject<Vec<T>> {
    fn write_array(&self, index: usize, val: T) {
        unsafe {
            let array = &mut *self.fields.get();
            assert!(index < array.len());
            array[index] = val;
        }
    }

    fn read_array(&self, index: usize) -> T {
        unsafe {
            let array = &*self.fields.get();
            assert!(index < array.len());
            array[index]
        }
    }

    fn array_length(&self) -> usize {
        unsafe {
            let array = &*self.fields.get();
            array.len()
        }
    }
}

impl<T: JavaPrimitive> RawObject<Vec<T>>
    where
        Self: Trace,
{
    pub fn array_copy(&self, dst: ObjectHandle, src_pos: usize, dst_pos: usize, len: usize) {
        let dst_array = dst.expect_array::<T>();

        unsafe {
            let src_vec = &*self.fields.get();
            let dst_vec = &mut *dst_array.deref().fields.get();
            dst_vec[dst_pos..dst_pos + len].copy_from_slice(&src_vec[src_pos..src_pos + len]);
        }

        //
        // for offset in 0..length as usize {
        //     dst_vec[dst_pos as usize + offset] = src_vec[src_pos as usize + offset].clone();
        // }
    }
}

impl<T: JavaPrimitive> Deref for RawObject<Vec<T>>
    where
        RawObject<Vec<T>>: ArrayReference<T>,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.fields.get() }
    }
}

impl<T: JavaPrimitive> DerefMut for RawObject<Vec<T>>
    where
        RawObject<Vec<T>>: ArrayReference<T>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.fields.get() }
    }
}


/// Utility trait to match type ids
pub trait ConstTypeId {
    const ID: TypeId;
}

impl<T: ?Sized + 'static> ConstTypeId for T {
    const ID: TypeId = TypeId::of::<Self>();
}