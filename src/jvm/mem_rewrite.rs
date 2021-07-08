use std::any::{type_name, TypeId};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::rc::Rc;

use crate::class::Class;
use crate::jvm::{LocalVariable, JVM};
use crate::types::FieldDescriptor;
use hashbrown::HashMap;
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort, jvalue};
use lazy_static::lazy_static;
use std::mem::{size_of, transmute};
use std::sync::Arc;

pub trait JavaPrimitive: 'static + Sized + Copy {
    fn pack(self) -> jvalue;
    fn unpack(val: jvalue) -> Self;

    fn descriptor() -> FieldDescriptor;
}

macro_rules! define_primitive {
    ($name:ty: $ref:ident, $fd:ident) => {
        impl JavaPrimitive for $name {
            fn pack(self) -> jvalue {
                jvalue { $ref: self }
            }

            fn unpack(val: jvalue) -> Self {
                unsafe { val.$ref }
            }
            fn descriptor() -> FieldDescriptor {
                FieldDescriptor::$fd
            }
        }
    };
}

define_primitive!(jboolean: z, Boolean);
define_primitive!(jbyte: b, Byte);
define_primitive!(jchar: c, Char);
define_primitive!(jshort: s, Short);
define_primitive!(jint: i, Int);
define_primitive!(jlong: j, Long);
define_primitive!(jfloat: f, Float);
define_primitive!(jdouble: d, Double);

#[derive(Clone)]
pub struct ObjectWrapper<T> {
    ptr: Pin<Rc<T>>,
}

impl<T> ObjectWrapper<T> {
    fn new(val: T) -> Self {
        ObjectWrapper { ptr: Rc::pin(val) }
    }

    #[inline]
    pub fn into_raw(self) -> jobject {
        unsafe { Rc::into_raw(Pin::into_inner_unchecked(self.ptr)) as jobject }
    }

    #[inline]
    pub unsafe fn from_raw_unchecked(ptr: jobject) -> Self {
        ObjectWrapper {
            ptr: Pin::new_unchecked(Rc::from_raw(ptr as _)),
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

impl<T> ObjectReference for ObjectWrapper<RawObject<T>> {
    fn get_class_schema(&self) -> Arc<ClassSchema> {
        self.ptr.schema.clone()
    }
}

impl<T> Deref for ObjectWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.ptr
    }
}

// TODO: Impl Hash
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct ObjectHandle(jobject);

impl From<jobject> for ObjectHandle {
    fn from(v: jobject) -> Self {
        ObjectHandle(v)
    }
}

impl JavaPrimitive for ObjectHandle {
    fn pack(self) -> jvalue {
        jvalue { l: self.0 }
    }

    fn unpack(val: jvalue) -> Self {
        unsafe { ObjectHandle(val.l) }
    }

    fn descriptor() -> FieldDescriptor {
        FieldDescriptor::Object("java/lang/Object".to_string())
    }
}

impl ObjectHandle {
    #[inline]
    pub fn unwrap_unknown(self) -> ObjectWrapper<RawObject<()>> {
        let ObjectHandle(ptr) = self;
        unsafe { ObjectWrapper::from_raw(ptr).unwrap() }
    }

    pub fn expect_instance(self) -> ObjectWrapper<RawObject<Vec<jvalue>>> {
        if self.memory_layout() != ObjectType::Instance {
            panic!("Expected invalid primitive array");
        }

        unsafe { transmute(self.unwrap_unknown()) }
    }

    pub fn expect_array<T: JavaPrimitive>(self) -> ObjectWrapper<RawObject<Vec<T>>> {
        if self.memory_layout() != ObjectType::Array(TypeId::of::<T>()) {
            panic!("Expected invalid primitive array");
        }

        unsafe { transmute(self.unwrap_unknown()) }
    }
}

impl ObjectReference for ObjectHandle {
    fn get_class_schema(&self) -> Arc<ClassSchema> {
        self.unwrap_unknown().get_class_schema()
    }
}

pub struct RawObject<T: ?Sized> {
    schema: Arc<ClassSchema>,
    fields: UnsafeCell<T>,
}

impl RawObject<Vec<jvalue>> {
    pub fn new(schema: Arc<ClassSchema>) -> Self {
        assert!(schema.data_form == ObjectType::Instance);
        RawObject {
            fields: UnsafeCell::new(vec![jvalue { j: 0 }; schema.field_offsets.len()]),
            schema,
        }
    }

    pub fn field_offset<S: AsRef<str>>(&self, field: S) -> usize {
        self.schema.field_offset(field)
    }
}

// TODO: Implement DST fields [jvalue] for raw object
// impl RawObject<[u64]> {
//     pub fn alloc(len: usize) {
//         let array = Layout::array::<u64>(len).unwrap();
//         let layout = Layout::new::<RawObject<()>>().extend(array).unwrap().0;
//     }
// }

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

impl<T: JavaPrimitive> InstanceReference<T> for RawObject<Vec<jvalue>> {
    fn write_field(&self, offset: usize, val: T) {
        self.write_field(offset, val.pack())
    }

    fn read_field(&self, offset: usize) -> T {
        T::unpack(self.read_field(offset))
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

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ObjectType {
    Instance,
    Array(TypeId),
}

impl ObjectType {
    pub fn is_array(&self) -> bool {
        matches!(self, ObjectType::Array(_))
    }

    pub fn is_instance(&self) -> bool {
        matches!(self, ObjectType::Instance)
    }

    pub fn is_array_of<T: 'static + ?Sized>(&self) -> bool {
        if let ObjectType::Array(id) = *self {
            return id == TypeId::of::<T>();
        }
        false
    }
}

// TODO: Add pseudo-vtable
pub struct ClassSchema {
    name: String,
    data_form: ObjectType,
    super_class: Option<Arc<ClassSchema>>,
    field_offsets: HashMap<String, usize>,
}

impl ClassSchema {
    pub fn build(class: &Class, jvm: &mut JVM) -> Self {
        let name = class.name();
        let mut field_offsets = HashMap::new();

        let super_class = match name.as_ref() {
            "java/lang/Object" => None,
            _ => jvm.class_schema(&class.super_class()),
        };

        if let Some(schema) = &super_class {
            field_offsets.extend(schema.field_offsets.iter().map(|(x, y)| (x.clone(), *y)));
        }

        ClassSchema {
            name,
            data_form: ObjectType::Instance,
            super_class,
            field_offsets,
        }
    }

    pub fn is_array(&self) -> bool {
        matches!(self.data_form, ObjectType::Array(_))
    }

    pub fn is_instance(&self) -> bool {
        matches!(self.data_form, ObjectType::Instance)
    }

    pub fn field_offset<S: AsRef<str>>(&self, field: S) -> usize {
        assert!(self.is_instance());

        match self.field_offsets.get(field.as_ref()) {
            Some(v) => *v,
            None => panic!(
                "Object {} does not have field: {:?}",
                self.name,
                field.as_ref()
            ),
        }
    }
}

// Work around to implement for Rc<ClassSchema>
// pub trait ObjectBuilder {
//     fn new(&self) -> ObjectHandle;
// }

// impl ObjectBuilder for Arc<ClassSchema> {
//
// }

impl ClassSchema {
    pub fn array_schema<T: JavaPrimitive>() -> Arc<ClassSchema> {
        match TypeId::of::<T>() {
            jboolean::ID => ARRAY_BOOL_SCHEMA.clone(),
            jbyte::ID => ARRAY_BYTE_SCHEMA.clone(),
            jchar::ID => ARRAY_CHAR_SCHEMA.clone(),
            jshort::ID => ARRAY_SHORT_SCHEMA.clone(),
            jint::ID => ARRAY_INT_SCHEMA.clone(),
            jlong::ID => ARRAY_LONG_SCHEMA.clone(),
            jfloat::ID => ARRAY_FLOAT_SCHEMA.clone(),
            jdouble::ID => ARRAY_DOUBLE_SCHEMA.clone(),
            ObjectHandle::ID => ARRAY_OBJECT_SCHEMA.clone(),
            _ => panic!("Unable to get array schema for {}", type_name::<T>()),
        }
    }

    fn init_array_schema<T: JavaPrimitive>() -> ClassSchema {
        ClassSchema {
            name: FieldDescriptor::Array(Box::new(T::descriptor())).to_string(),
            data_form: ObjectType::Array(TypeId::of::<T>()),
            super_class: Some(OBJECT_SCHEMA.clone()),
            field_offsets: HashMap::new(),
        }
    }
}

lazy_static! {
    pub static ref OBJECT_SCHEMA: Arc<ClassSchema> = Arc::new(ClassSchema {
        name: "java/lang/Object".to_string(),
        data_form: ObjectType::Instance,
        super_class: None,
        field_offsets: HashMap::new(),
    });
}

macro_rules! array_schema {
    ($name:ident: $type:ty, $fd:literal) => {
        lazy_static! {
            pub static ref $name: Arc<ClassSchema> = Arc::new(ClassSchema {
                name: $fd.to_string(),
                data_form: ObjectType::Array(TypeId::of::<$type>()),
                super_class: Some(OBJECT_SCHEMA.clone()),
                field_offsets: HashMap::new(),
            });
        }
    };
}

array_schema!(ARRAY_BOOL_SCHEMA: jboolean, "[Z");
array_schema!(ARRAY_BYTE_SCHEMA: jbyte, "[B");
array_schema!(ARRAY_CHAR_SCHEMA: jchar, "[C");
array_schema!(ARRAY_SHORT_SCHEMA: jshort, "[S");
array_schema!(ARRAY_INT_SCHEMA: jint, "[I");
array_schema!(ARRAY_LONG_SCHEMA: jlong, "[J");
array_schema!(ARRAY_FLOAT_SCHEMA: jfloat, "[F");
array_schema!(ARRAY_DOUBLE_SCHEMA: jdouble, "[D");
array_schema!(ARRAY_OBJECT_SCHEMA: ObjectHandle, "[Ljava/lang/Object;");

// Work around to match type ids
trait ConstTypeId {
    const ID: TypeId;
}

impl<T: ?Sized + 'static> ConstTypeId for T {
    const ID: TypeId = TypeId::of::<Self>();
}

impl ObjectHandle {
    /// Allocates a new zeroed object instance
    fn new(schema: Arc<ClassSchema>) -> ObjectHandle {
        ObjectHandle(ObjectWrapper::new(RawObject::new(schema)).into_raw())
    }

    pub fn array_from_data<T: JavaPrimitive>(arr: Vec<T>) -> ObjectHandle {
        let raw = RawObject {
            schema: ClassSchema::array_schema::<T>(),
            fields: UnsafeCell::new(arr),
        };

        ObjectHandle(ObjectWrapper::new(raw).into_raw())
    }

    pub fn from_fields<S: AsRef<str>>(
        schema: Arc<ClassSchema>,
        fields: HashMap<S, LocalVariable>,
    ) -> ObjectHandle {
        let raw = RawObject::new(schema);

        for (field, value) in fields {
            let offset = raw.field_offset(field);
            InstanceReference::<jvalue>::write_field(&raw, offset, value.into());
        }

        ObjectHandle(ObjectWrapper::new(raw).into_raw())
    }
}
