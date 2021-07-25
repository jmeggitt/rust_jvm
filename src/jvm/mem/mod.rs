use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::mem::transmute_copy;

use jni::sys::{jvalue, jbyte, jchar, jshort, jint, jfloat, jlong, jdouble};

use std::fmt::{Debug, Formatter};

pub use handle::*;
pub use raw::*;
pub use schema::*;
pub use types::*;

mod handle;
mod raw;
mod schema;
mod types;

/// All distinct java values
#[derive(Copy, Clone)]
pub enum JavaValue {
    Byte(jbyte),
    Char(jchar),
    Short(jshort),
    Int(jint),
    Float(jfloat),
    Long(jlong),
    Double(jdouble),
    Reference(Option<ObjectHandle>),
}

impl Hash for JavaValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            JavaValue::Byte(x) => x.hash(state),
            JavaValue::Char(x) => x.hash(state),
            JavaValue::Short(x) => x.hash(state),
            JavaValue::Int(x) => x.hash(state),
            JavaValue::Float(x) => unsafe { transmute_copy::<_, u32>(x).hash(state) },
            JavaValue::Reference(Some(reference)) => reference.hash(state),
            // JavaValue::Reference(Some(reference)) => unsafe { (&*reference.get()).hash(state) },
            JavaValue::Long(x) => x.hash(state),
            JavaValue::Double(x) => unsafe { transmute_copy::<_, u64>(x).hash(state) },
            _ => {}
        }
    }
}

impl JavaValue {
    /// Helper function for conversion during match operations
    pub fn as_int(&self) -> Option<i64> {
        Some(match self {
            JavaValue::Byte(x) => *x as i64,
            // FIXME: I'm unsure if this is technically correct or not
            JavaValue::Char(x) => unsafe { ::std::mem::transmute::<_, i16>(*x) as i64 },
            JavaValue::Short(x) => *x as i64,
            JavaValue::Int(x) => *x as i64,
            JavaValue::Long(x) => *x as i64,
            _ => return None,
        })
    }

    /// Helper function for conversion during match operations
    pub fn as_float(&self) -> Option<f64> {
        Some(match self {
            JavaValue::Byte(x) => *x as f64,
            // FIXME: I'm unsure if this is technically correct or not
            JavaValue::Char(x) => unsafe { ::std::mem::transmute::<_, i16>(*x) as f64 },
            JavaValue::Short(x) => *x as f64,
            JavaValue::Int(x) => *x as f64,
            JavaValue::Long(x) => *x as f64,
            JavaValue::Float(x) => *x as f64,
            JavaValue::Double(x) => *x as f64,
            _ => return None,
        })
    }

    pub fn signum(&self) -> Option<i32> {
        Some(match self {
            JavaValue::Byte(x) => x.signum() as i32,
            JavaValue::Char(x) if *x == 0 => 0,
            JavaValue::Char(_) => 1,
            JavaValue::Short(x) => x.signum() as i32,
            JavaValue::Int(x) => x.signum() as i32,
            JavaValue::Float(x) => x.signum() as i32,
            JavaValue::Long(x) => x.signum() as i32,
            JavaValue::Double(x) => x.signum() as i32,
            _ => return None,
        })
    }
}

impl From<ObjectHandle> for JavaValue {
    fn from(obj: ObjectHandle) -> Self {
        JavaValue::Reference(Some(obj))
    }
}

impl PartialEq for JavaValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JavaValue::Byte(a), JavaValue::Byte(b)) => a.eq(b),
            (JavaValue::Char(a), JavaValue::Char(b)) => a.eq(b),
            (JavaValue::Short(a), JavaValue::Short(b)) => a.eq(b),
            (JavaValue::Int(a), JavaValue::Int(b)) => a.eq(b),
            (JavaValue::Float(a), JavaValue::Float(b)) => a.eq(b),
            (JavaValue::Long(a), JavaValue::Long(b)) => a.eq(b),
            (JavaValue::Double(a), JavaValue::Double(b)) => a.eq(b),
            _ => false,
        }
    }
}

impl PartialOrd for JavaValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            // (JavaValue::Byte(a), JavaValue::Byte(b)) => a.partial_cmp(b),
            // (JavaValue::Char(a), JavaValue::Char(b)) => a.partial_cmp(b),
            // (JavaValue::Short(a), JavaValue::Short(b)) => a.partial_cmp(b),
            // (JavaValue::Int(a), JavaValue::Int(b)) => a.partial_cmp(b),
            // (JavaValue::Long(a), JavaValue::Long(b)) => a.partial_cmp(b),
            (JavaValue::Float(a), JavaValue::Float(b)) => a.partial_cmp(b),
            (JavaValue::Double(a), JavaValue::Double(b)) => a.partial_cmp(b),
            (JavaValue::Float(a), JavaValue::Double(b)) => (*a as f64).partial_cmp(b),
            (JavaValue::Double(a), JavaValue::Float(b)) => a.partial_cmp(&(*b as f64)),
            // (JavaValue::Reference(Some(a)), JavaValue::Reference(Some(b))) => {
            //     a.get().partial_cmp(&b.get())
            // }
            (JavaValue::Reference(Some(_)), JavaValue::Reference(_)) => {
                Some(Ordering::Greater)
            }
            (JavaValue::Reference(_), JavaValue::Reference(Some(_))) => {
                Some(Ordering::Less)
            }
            (JavaValue::Reference(_), JavaValue::Reference(_)) => Some(Ordering::Equal),
            (a, b) => match (a.as_int(), b.as_int()) {
                (Some(x), Some(y)) => x.partial_cmp(&y),
                _ => None,
            },
        }
    }
}

impl Into<jvalue> for JavaValue {
    fn into(self) -> jvalue {
        match self {
            JavaValue::Byte(x) => jvalue { b: x },
            JavaValue::Char(x) => jvalue { c: x },
            JavaValue::Short(x) => jvalue { s: x },
            JavaValue::Int(x) => jvalue { i: x },
            JavaValue::Float(x) => jvalue { f: x },
            JavaValue::Reference(x) => x.pack(),
            JavaValue::Long(x) => jvalue { j: x },
            JavaValue::Double(x) => jvalue { d: x },
        }
    }
}

impl Debug for JavaValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JavaValue::Byte(x) => write!(f, "{}u8", x),
            JavaValue::Char(x) => write!(f, "{:?}", std::char::from_u32(*x as u32).unwrap()),
            JavaValue::Short(x) => write!(f, "{}i16", x),
            JavaValue::Int(x) => write!(f, "{}i32", x),
            JavaValue::Float(x) => write!(f, "{}f32", x),
            JavaValue::Reference(x) => write!(f, "{:?}", x),
            JavaValue::Long(x) => write!(f, "{}i64", x),
            JavaValue::Double(x) => write!(f, "{}f64", x),
        }
    }
}
