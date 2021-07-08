use std::alloc;
use std::alloc::Layout;
use std::cell::{UnsafeCell, Cell};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr;
use std::any::{TypeId, Any, type_name};
use std::rc::Rc;

use hashbrown::HashMap;
use lazy_static::lazy_static;
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort, jvalue};
use std::mem::{size_of, transmute};
use crate::types::FieldDescriptor;
use crate::jvm::JVM;
use crate::class::Class;
use std::sync::Arc;
use std::fmt::{Debug, Formatter};

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
        ObjectWrapper {
            ptr: Rc::pin(val),
        }
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
        unsafe {
            ObjectHandle(val.l)
        }
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

// #[repr(C, packed)]
// TODO: Replace fields with DST [u64]
pub struct RawObject<T: ?Sized> {
    schema: Arc<ClassSchema>,
    fields: UnsafeCell<T>,
}


impl RawObject<[u64]> {
    pub fn alloc(len: usize) {
        unsafe {
            // let offset = &(*(ptr::null::<Self>())).fields as *const _ as usize;

            // Find length of zero-element fields (probably mem::size_of::<usize>()).
            // let base_size = mem::size_of_val(&([0u64; 0] as [u64]));
            // let size = offset + base_size + len * mem::size_of::<u64>();

            // Align should be 0, but check just in case
            // let layout = alloc::Layout::from_size_align(size, mem::align_of::<Self>()).unwrap();

            let array = Layout::array::<u64>(len).unwrap();
            let layout = Layout::new::<RawObject<()>>().extend(array).unwrap().0;

            // let raw = alloc::alloc(layout);
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

pub trait InstanceReference<T: JavaPrimitive>: ObjectReference {
    fn write_field(&self, offset: usize, val: T);
    fn read_field(&self, offset: usize) -> T;
}

impl<T: JavaPrimitive> InstanceReference<T> for RawObject<Vec<jvalue>> {
    fn write_field(&self, offset: usize, val: T) {
        assert_eq!(offset % size_of::<jvalue>(), 0);
        let index = offset / size_of::<jvalue>();

        unsafe {
            let fields = &mut *self.fields.get();
            assert!(index < fields.len());
            fields[index] = val.pack();
        }
    }

    fn read_field(&self, offset: usize) -> T {
        assert_eq!(offset % size_of::<jvalue>(), 0);
        let index = offset / size_of::<jvalue>();

        unsafe {
            let fields = &*self.fields.get();
            assert!(index < fields.len());
            T::unpack(fields[index])
        }
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
    where RawObject<Vec<T>>: ArrayReference<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.fields.get() }
    }
}

impl<T: JavaPrimitive> DerefMut for RawObject<Vec<T>>
    where RawObject<Vec<T>>: ArrayReference<T> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.fields.get() }
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ObjectType {
    Instance,
    Array(TypeId),
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

}

// Work around to implement for Rc<ClassSchema>
pub trait ObjectBuilder {
    fn new(&self) -> ObjectHandle;
}

impl ObjectBuilder for Arc<ClassSchema> {
    /// Allocates a new zeroed object instance
    fn new(&self) -> ObjectHandle {
        assert!(self.data_form == ObjectType::Instance);

        let raw = RawObject {
            schema: self.clone(),
            fields: UnsafeCell::new(vec![jvalue { j: 0 }; self.field_offsets.len()]),
        };
        unsafe { ObjectHandle(ObjectWrapper::new(raw).into_raw()) }
    }
}

impl ClassSchema {
    pub fn for_array<T: JavaPrimitive>() -> ClassSchema {
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

pub fn get_array_schema<T: JavaPrimitive>() -> Arc<ClassSchema> {
    trait ConstTypeId {
        const ID: TypeId;
    }

    impl<T: JavaPrimitive> ConstTypeId for T {
        const ID: TypeId = TypeId::of::<Self>();
    }

    match TypeId::of::<T>() {
        jboolean::ID => ARRAY_BOOL_SCHEMA.clone(),

        x => panic!("Unable to get array schema for {}", type_name::<T>()),
    }
}

pub fn array_from_data<T: JavaPrimitive>(arr: Vec<T>) -> ObjectHandle {
    // FIXME: Be lazy and make a new schema each time
    todo!()
}
