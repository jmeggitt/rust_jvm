use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::mem::transmute_copy;
use std::ptr::null_mut;
use std::rc::Rc;

use hashbrown::HashMap;
use jni::sys::{jobject, jvalue};

use crate::jvm::JVM;
use crate::types::FieldDescriptor;
use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone)]
pub enum Object {
    Instance {
        fields: HashMap<String, LocalVariable>,
        class: String,
    },
    Class(String),
    Array {
        values: Vec<LocalVariable>,
        element_type: FieldDescriptor,
    },
    Box(LocalVariable),
}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Object::Instance { fields, class } => {
                class.hash(state);
                let mut keys: Vec<String> = fields.keys().cloned().collect();
                keys.sort();
                for key in keys {
                    key.hash(state);
                    fields.get(&key).hash(state);
                }
            }
            Object::Class(name) => name.hash(state),
            Object::Array {
                values,
                element_type,
            } => {
                element_type.hash(state);
                values.hash(state);
            }
            Object::Box(value) => value.hash(state),
        }
    }
}

impl Object {
    pub fn expect_string(&self) -> String {
        if let Object::Instance { fields, .. } = self {
            if let Some(LocalVariable::Reference(Some(data))) = fields.get("value") {
                if let Object::Array { values, .. } = unsafe { &*data.get() } {
                    let mut bytes = Vec::with_capacity(values.len());

                    for val in values {
                        if let LocalVariable::Char(v) = val {
                            bytes.push(*v as u8);
                        }
                    }

                    String::from_utf8(bytes).unwrap()
                } else {
                    panic!("Data field of string was not an array")
                }
            } else {
                panic!("Did not have data field")
            }
        } else {
            panic!("Attempted to read string from non-instance")
        }
    }

    pub fn build_class(jvm: &mut JVM, name: &str) -> Rc<UnsafeCell<Self>> {
        jvm.init_class("java/lang/Class");

        let base_obj = Rc::new(UnsafeCell::new(Object::Instance {
            fields: HashMap::new(),
            class: "java/lang/Class".to_string(),
        }));

        // jvm.exec_method(base_obj.clone(), "<init>", "()V", vec![])
        //     .unwrap();
        if let Self::Instance { fields, .. } = unsafe { &mut *base_obj.get() } {
            fields.insert("name".to_string(), name.into());
            fields.insert("classLoader".to_string(), LocalVariable::Reference(None));
        }
        base_obj
    }

    pub fn expect_class(&self) -> Option<String> {
        match self {
            Object::Instance { class, .. } => Some(class.to_string()),
            Object::Class(name) => Some(name.to_string()),
            Object::Array { element_type, .. } => Some(format!("[{}", element_type.to_string())),
            _ => None,
        }
    }

    pub fn build_byte_array(arr: &str) -> Object {
        let mut bytes = Vec::with_capacity(arr.len());

        for byte in arr.bytes() {
            bytes.push(LocalVariable::Byte(byte as i8));
        }

        Object::Array {
            values: bytes,
            element_type: FieldDescriptor::Byte,
        }
    }
}

impl From<&str> for Object {
    fn from(src: &str) -> Self {
        let mut chars = Vec::with_capacity(src.len());

        for char in src.chars() {
            chars.push(LocalVariable::Char(char as u16));
        }

        Object::Array {
            values: chars,
            element_type: FieldDescriptor::Char,
        }
    }
}

impl<'a> Into<jobject> for &'a Object {
    fn into(self) -> jobject {
        self as *const _ as jobject
    }
}

/// All distinct java values
#[derive(Clone)]
pub enum LocalVariable {
    Byte(i8),
    Char(u16),
    Short(i16),
    Int(i32),
    Float(f32),
    Reference(Option<Rc<UnsafeCell<Object>>>),
    Long(i64),
    Double(f64),
}

impl Hash for LocalVariable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            LocalVariable::Byte(x) => x.hash(state),
            LocalVariable::Char(x) => x.hash(state),
            LocalVariable::Short(x) => x.hash(state),
            LocalVariable::Int(x) => x.hash(state),
            LocalVariable::Float(x) => unsafe { transmute_copy::<_, u32>(x).hash(state) },
            LocalVariable::Reference(Some(reference)) => unsafe { (&*reference.get()).hash(state) },
            LocalVariable::Long(x) => x.hash(state),
            LocalVariable::Double(x) => unsafe { transmute_copy::<_, u64>(x).hash(state) },
            _ => {}
        }
    }
}

impl LocalVariable {
    /// Helper function for conversion during match operations
    pub fn as_int(&self) -> Option<i64> {
        Some(match self {
            LocalVariable::Byte(x) => *x as i64,
            // FIXME: I'm unsure if this is technically correct or not
            LocalVariable::Char(x) => unsafe { ::std::mem::transmute::<_, i16>(*x) as i64 },
            LocalVariable::Short(x) => *x as i64,
            LocalVariable::Int(x) => *x as i64,
            LocalVariable::Long(x) => *x as i64,
            _ => return None,
        })
    }

    /// Helper function for conversion during match operations
    pub fn as_float(&self) -> Option<f64> {
        Some(match self {
            LocalVariable::Byte(x) => *x as f64,
            // FIXME: I'm unsure if this is technically correct or not
            LocalVariable::Char(x) => unsafe { ::std::mem::transmute::<_, i16>(*x) as f64 },
            LocalVariable::Short(x) => *x as f64,
            LocalVariable::Int(x) => *x as f64,
            LocalVariable::Long(x) => *x as f64,
            LocalVariable::Float(x) => *x as f64,
            LocalVariable::Double(x) => *x as f64,
            _ => return None,
        })
    }

    pub fn signum(&self) -> Option<i32> {
        Some(match self {
            LocalVariable::Byte(x) => x.signum() as i32,
            LocalVariable::Char(x) if *x == 0 => 0,
            LocalVariable::Char(_) => 1,
            LocalVariable::Short(x) => x.signum() as i32,
            LocalVariable::Int(x) => x.signum() as i32,
            LocalVariable::Float(x) => x.signum() as i32,
            LocalVariable::Long(x) => x.signum() as i32,
            LocalVariable::Double(x) => x.signum() as i32,
            _ => return None,
        })
    }
}

impl From<Object> for LocalVariable {
    fn from(obj: Object) -> Self {
        LocalVariable::Reference(Some(Rc::new(UnsafeCell::new(obj))))
    }
}

impl From<&str> for LocalVariable {
    fn from(string: &str) -> Self {
        let mut fields = HashMap::new();

        fields.insert("value".to_string(), Object::build_byte_array(string).into());

        // Claim encoding is Latin1
        fields.insert("coder".to_string(), LocalVariable::Byte(0));

        // Hashing information
        fields.insert("hash".to_string(), LocalVariable::Int(0));
        fields.insert("hashIsZero".to_string(), LocalVariable::Byte(0));

        LocalVariable::Reference(Some(Rc::new(UnsafeCell::new(Object::Instance {
            fields,
            class: "java/lang/String".to_string(),
        }))))
    }
}

impl PartialEq for LocalVariable {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LocalVariable::Byte(a), LocalVariable::Byte(b)) => a.eq(b),
            (LocalVariable::Char(a), LocalVariable::Char(b)) => a.eq(b),
            (LocalVariable::Short(a), LocalVariable::Short(b)) => a.eq(b),
            (LocalVariable::Int(a), LocalVariable::Int(b)) => a.eq(b),
            (LocalVariable::Float(a), LocalVariable::Float(b)) => a.eq(b),
            (LocalVariable::Long(a), LocalVariable::Long(b)) => a.eq(b),
            (LocalVariable::Double(a), LocalVariable::Double(b)) => a.eq(b),
            _ => false,
        }
    }
}

impl PartialOrd for LocalVariable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            // (LocalVariable::Byte(a), LocalVariable::Byte(b)) => a.partial_cmp(b),
            // (LocalVariable::Char(a), LocalVariable::Char(b)) => a.partial_cmp(b),
            // (LocalVariable::Short(a), LocalVariable::Short(b)) => a.partial_cmp(b),
            // (LocalVariable::Int(a), LocalVariable::Int(b)) => a.partial_cmp(b),
            // (LocalVariable::Long(a), LocalVariable::Long(b)) => a.partial_cmp(b),
            (LocalVariable::Float(a), LocalVariable::Float(b)) => a.partial_cmp(b),
            (LocalVariable::Double(a), LocalVariable::Double(b)) => a.partial_cmp(b),
            (LocalVariable::Float(a), LocalVariable::Double(b)) => (*a as f64).partial_cmp(b),
            (LocalVariable::Double(a), LocalVariable::Float(b)) => a.partial_cmp(&(*b as f64)),
            (LocalVariable::Reference(Some(a)), LocalVariable::Reference(Some(b))) => {
                a.get().partial_cmp(&b.get())
            }
            (LocalVariable::Reference(Some(_)), LocalVariable::Reference(_)) => {
                Some(Ordering::Greater)
            }
            (LocalVariable::Reference(_), LocalVariable::Reference(Some(_))) => {
                Some(Ordering::Less)
            }
            (LocalVariable::Reference(_), LocalVariable::Reference(_)) => Some(Ordering::Equal),
            (a, b) => match (a.as_int(), b.as_int()) {
                (Some(x), Some(y)) => x.partial_cmp(&y),
                _ => None,
            },
        }
    }
}

impl Into<jvalue> for LocalVariable {
    fn into(self) -> jvalue {
        match self {
            LocalVariable::Byte(x) => jvalue { b: x },
            LocalVariable::Char(x) => jvalue { c: x },
            LocalVariable::Short(x) => jvalue { s: x },
            LocalVariable::Int(x) => jvalue { i: x },
            LocalVariable::Float(x) => jvalue { f: x },
            LocalVariable::Reference(None) => jvalue { l: null_mut() },
            LocalVariable::Reference(Some(x)) => jvalue {
                l: Rc::into_raw(x) as jobject,
            },
            LocalVariable::Long(x) => jvalue { j: x },
            LocalVariable::Double(x) => jvalue { d: x },
        }
    }
}

impl Debug for LocalVariable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalVariable::Byte(x) => write!(f, "{}", x),
            LocalVariable::Char(x) => write!(f, "{:?}", std::char::from_u32(*x as u32).unwrap()),
            LocalVariable::Short(x) => write!(f, "{}", x),
            LocalVariable::Int(x) => write!(f, "{}", x),
            LocalVariable::Float(x) => write!(f, "{}", x),
            LocalVariable::Reference(x) => write!(f, "{:?}", x),
            LocalVariable::Long(x) => write!(f, "{}", x),
            LocalVariable::Double(x) => write!(f, "{}", x),
        }
    }
}
