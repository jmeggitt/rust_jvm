use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::mem::transmute_copy;
use std::ptr::null_mut;
use std::rc::Rc;

use hashbrown::HashMap;
use jni::sys::{jbyte, jobject, jvalue, JNI_FALSE};

use crate::jvm::mem_rewrite::STRING_SCHEMA;
use crate::jvm::mem_rewrite::{JavaPrimitive, ManualInstanceReference, ObjectHandle};
use crate::jvm::JVM;
use crate::types::FieldDescriptor;
use std::fmt::{Debug, Formatter};

/// All distinct java values
#[derive(Clone)]
pub enum LocalVariable {
    Byte(i8),
    Char(u16),
    Short(i16),
    Int(i32),
    Float(f32),
    Reference(Option<ObjectHandle>),
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
            LocalVariable::Reference(Some(reference)) => reference.hash(state),
            // LocalVariable::Reference(Some(reference)) => unsafe { (&*reference.get()).hash(state) },
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

// impl From<Object> for LocalVariable {
//     fn from(obj: Object) -> Self {
//         LocalVariable::Reference(Some(Rc::new(UnsafeCell::new(obj))))
//     }
// }

impl From<ObjectHandle> for LocalVariable {
    fn from(obj: ObjectHandle) -> Self {
        LocalVariable::Reference(Some(obj))
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
            // (LocalVariable::Reference(Some(a)), LocalVariable::Reference(Some(b))) => {
            //     a.get().partial_cmp(&b.get())
            // }
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
            LocalVariable::Reference(Some(x)) => Some(x).pack(),
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
