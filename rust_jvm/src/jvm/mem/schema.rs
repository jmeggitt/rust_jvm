use crate::class::{AccessFlags, BufferedRead, Class};
// use crate::jvm::call::interface::GLOBAL_JVM;
use crate::jvm::mem::{ConstTypeId, FieldDescriptor, JavaPrimitive, JavaTypeEnum, ObjectHandle};
use crate::jvm::JavaEnv;
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort, jvalue};
use lazy_static::lazy_static;
use std::any::type_name;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::mem::size_of;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct FieldSchema {
    pub offset: usize,
    pub name: String,
    pub desc: FieldDescriptor,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ObjectType {
    Instance,
    Array(JavaTypeEnum),
}

impl Debug for ObjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::Instance => write!(f, "Instance"),
            ObjectType::Array(jboolean::ID) => write!(f, "Array(jboolean)"),
            ObjectType::Array(jbyte::ID) => write!(f, "Array(jbyte)"),
            ObjectType::Array(jchar::ID) => write!(f, "Array(jchar)"),
            ObjectType::Array(jshort::ID) => write!(f, "Array(jshort)"),
            ObjectType::Array(jint::ID) => write!(f, "Array(jint)"),
            ObjectType::Array(jlong::ID) => write!(f, "Array(jlong)"),
            ObjectType::Array(jfloat::ID) => write!(f, "Array(jfloat)"),
            ObjectType::Array(jdouble::ID) => write!(f, "Array(jdouble)"),
            ObjectType::Array(<Option<ObjectHandle> as ConstTypeId>::ID) => {
                write!(f, "Array(jobject)")
            }
            ObjectType::Array(x) => write!(f, "Array({:?})", x),
        }
    }
}

impl ObjectType {
    pub fn is_array(&self) -> bool {
        matches!(self, ObjectType::Array(_))
    }

    pub fn is_instance(&self) -> bool {
        matches!(self, ObjectType::Instance)
    }

    pub fn is_array_of<T: 'static + ?Sized + JavaPrimitive>(&self) -> bool {
        if let ObjectType::Array(id) = *self {
            return id == T::ID;
        }
        false
    }
}

#[derive(Debug)]
pub struct ClassSchema {
    pub name: String,
    pub data_form: ObjectType,
    pub super_class: Option<Arc<ClassSchema>>,
    pub field_offsets: HashMap<String, FieldSchema>,
    pub field_lookup: Vec<FieldSchema>,
}

impl ClassSchema {
    /// Performs instanceof on this class. This excludes interfaces when searching to avoid
    /// additional lookup.
    ///
    /// Note: I wonder if Rust is able to perform tail recursion with this code? I think I saw
    /// something about it in one of the more recent release notes.
    pub fn direct_instanceof(&self, of: &str) -> bool {
        if self.name == of {
            return true;
        }

        match self.super_class.as_ref() {
            Some(v) => v.direct_instanceof(of),
            None => false,
        }
    }

    pub fn build(class: &Class, jvm: &mut JavaEnv) -> Self {
        let name = class.name();
        debug!("Building new schema for {}", &name);

        let super_class = match name.as_ref() {
            "java/lang/Object" => None,
            _ => Some(jvm.class_schema(&class.super_class())),
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

            if field_offsets.contains_key(name) {
                continue;
            }

            field_offsets.insert(name.to_string(), field.clone());
            field_lookup.push(field);
        }

        if field_offsets.len() != field_lookup.len() {
            error!("Superclass schema: {:?}", &super_class);
            error!("Offsets: {:?}", &field_offsets);
            error!("Lookup: {:?}", &field_lookup);

            // Trigger panic
            assert_eq!(field_offsets.len(), field_lookup.len());
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

    #[track_caller]
    pub fn field_offset<S: AsRef<str>>(&self, field: S) -> usize {
        // assert_eq!(self.data_form, ObjectType::Instance);
        // assert!(self.is_instance());

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
        match T::ID {
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
            data_form: ObjectType::Array(T::ID),
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
    pub static ref ARRAY_BOOL_SCHEMA: Arc<ClassSchema> =
        Arc::new(ClassSchema::init_array_schema::<jboolean>());
    pub static ref ARRAY_BYTE_SCHEMA: Arc<ClassSchema> =
        Arc::new(ClassSchema::init_array_schema::<jbyte>());
    pub static ref ARRAY_CHAR_SCHEMA: Arc<ClassSchema> =
        Arc::new(ClassSchema::init_array_schema::<jchar>());
    pub static ref ARRAY_SHORT_SCHEMA: Arc<ClassSchema> =
        Arc::new(ClassSchema::init_array_schema::<jshort>());
    pub static ref ARRAY_INT_SCHEMA: Arc<ClassSchema> =
        Arc::new(ClassSchema::init_array_schema::<jint>());
    pub static ref ARRAY_LONG_SCHEMA: Arc<ClassSchema> =
        Arc::new(ClassSchema::init_array_schema::<jlong>());
    pub static ref ARRAY_FLOAT_SCHEMA: Arc<ClassSchema> =
        Arc::new(ClassSchema::init_array_schema::<jfloat>());
    pub static ref ARRAY_DOUBLE_SCHEMA: Arc<ClassSchema> =
        Arc::new(ClassSchema::init_array_schema::<jdouble>());
    pub static ref ARRAY_OBJECT_SCHEMA: Arc<ClassSchema> =
        Arc::new(ClassSchema::init_array_schema::<Option<ObjectHandle>>());
}
