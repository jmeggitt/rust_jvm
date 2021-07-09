use std::any::{type_name, TypeId};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::rc::Rc;

use crate::class::{Class, FieldInfo, AccessFlags, BufferedRead};
use crate::jvm::{LocalVariable, JVM};
use crate::types::FieldDescriptor;
use hashbrown::HashMap;
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort, jvalue, _jobject};
use lazy_static::lazy_static;
use std::mem::{size_of, transmute};
use std::sync::Arc;
use std::ptr::{null_mut, NonNull};
use std::hash::{Hash, Hasher};
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

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct ObjectHandle(NonNull<_jobject>);

impl JavaPrimitive for Option<ObjectHandle> {
    fn pack(self) -> jvalue {
        match self {
            Some(v) => jvalue { l: v.0.as_ptr() },
            None => jvalue {l: null_mut() },
        }
    }

    fn unpack(val: jvalue) -> Self {
        unsafe {
            match NonNull::new(val.l) {
                Some(v) => Some(ObjectHandle(v)),
                None => None,
            }
        }
    }

    fn descriptor() -> FieldDescriptor {
        FieldDescriptor::Object("java/lang/Object".to_string())
    }
}

impl ObjectHandle {
    pub fn from_ptr(x: jobject) -> Option<Self> {
        match NonNull::new(x) {
            Some(v) => Some(ObjectHandle(v)),
            None => None,
        }
    }

    #[inline]
    pub fn unwrap_unknown(self) -> ObjectWrapper<RawObject<()>> {
        let ObjectHandle(ptr) = self;
        unsafe { ObjectWrapper::from_raw(ptr.as_ptr()).unwrap() }
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

pub struct RawObject<T: ?Sized> {
    schema: Arc<ClassSchema>,
    fields: UnsafeCell<T>,
}

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
    where Self: ArrayReference<T> {

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
            ObjectType::Array(<Option<ObjectHandle> as ConstTypeId>::ID) => owned.expect_array::<Option<ObjectHandle>>().fmt(f),
            x => panic!("Unable to hash object of type {:?}", x),
        }
    }
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
    where Self: ArrayReference<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe {
            let fields = &*self.fields.get();
            fields.hash(state);
        }
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

impl InstanceReference<LocalVariable> for RawObject<Vec<jvalue>> {
    fn write_field(&self, offset: usize, val: LocalVariable) {
        let field = self.schema.get_field_from_offset(offset);
        assert!(field.desc.matches(&val));
        <Self as InstanceReference<jvalue>>::write_field(self, offset, val.into());
    }

    fn read_field(&self, offset: usize) -> LocalVariable {
        let field = self.schema.get_field_from_offset(offset);
        field.desc.cast(self.read_field(offset)).expect("field can not be cast to local")
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

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct FieldSchema {
    pub offset: usize,
    pub name: String,
    pub desc: FieldDescriptor,
}

pub struct ClassSchema {
    name: String,
    data_form: ObjectType,
    super_class: Option<Arc<ClassSchema>>,
    field_offsets: HashMap<String, FieldSchema>,
    field_lookup: Vec<FieldSchema>,
}

impl ClassSchema {
    pub fn build(class: &Class, jvm: &mut JVM) -> Self {
        let name = class.name();

        let super_class = match name.as_ref() {
            "java/lang/Object" => None,
            _ => jvm.class_schema(&class.super_class()),
        };

        let (mut field_offsets, mut field_lookup) = match &super_class {
           Some(v) => (v.field_offsets.clone(), v.field_lookup.clone()),
            None => Default::default(),
        };

        let pool = class.constants();
        for field in &class.fields {
            if field.access.contains(AccessFlags::STATIC) {
                continue;
            }

            let name = pool.text(field.name_index);
            let desc = pool.text(field.descriptor_index);
            let field = FieldSchema {
                offset: field_offsets.len() * size_of::<jvalue>(),
                name: name.to_string(),
                desc: FieldDescriptor::read_str(desc).expect("Unable to parse FieldDescriptor"),
            };

            field_offsets.insert(name.to_string(), field.clone());
            field_lookup.push(field);
        }

        ClassSchema {
            name,
            data_form: ObjectType::Instance,
            super_class,
            field_offsets,
            field_lookup,
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
            Some(v) => v.offset,
            None => panic!(
                "Object {} does not have field: {:?}",
                self.name,
                field.as_ref()
            ),
        }
    }

    pub fn get_field_from_offset(&self, offset: usize) -> &FieldSchema {
        assert_eq!(offset % size_of::<jvalue>(), 0);
        &self.field_lookup[offset / size_of::<jvalue>()]
    }
}

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
            <Option<ObjectHandle>>::ID => ARRAY_OBJECT_SCHEMA.clone(),
            _ => panic!("Unable to get array schema for {}", type_name::<T>()),
        }
    }

    fn init_array_schema<T: JavaPrimitive>() -> ClassSchema {
        ClassSchema {
            name: FieldDescriptor::Array(Box::new(T::descriptor())).to_string(),
            data_form: ObjectType::Array(TypeId::of::<T>()),
            super_class: Some(OBJECT_SCHEMA.clone()),
            field_offsets: HashMap::new(),
            field_lookup: Vec::new(),
        }
    }
}

lazy_static! {
    pub static ref OBJECT_SCHEMA: Arc<ClassSchema> = Arc::new(ClassSchema {
        name: "java/lang/Object".to_string(),
        data_form: ObjectType::Instance,
        super_class: None,
        field_offsets: HashMap::new(),
        field_lookup: Vec::new(),
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
                field_lookup: Vec::new(),
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
array_schema!(ARRAY_OBJECT_SCHEMA: Option<ObjectHandle>, "[Ljava/lang/Object;");

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
        ObjectHandle(NonNull::new(ObjectWrapper::new(RawObject::new(schema)).into_raw()).unwrap())
    }

    fn new_array<T: JavaPrimitive + Default>(len: usize) -> ObjectHandle {
        ObjectHandle::array_from_data(vec![T::default(); len])
    }

    pub fn array_from_data<T: JavaPrimitive>(arr: Vec<T>) -> ObjectHandle {
        let raw = RawObject {
            schema: ClassSchema::array_schema::<T>(),
            fields: UnsafeCell::new(arr),
        };

        ObjectHandle(NonNull::new(ObjectWrapper::new(raw).into_raw()).unwrap())
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

        ObjectHandle(NonNull::new(ObjectWrapper::new(raw).into_raw()).unwrap())
    }
}
