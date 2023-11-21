use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::mem::transmute_copy;

use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort, jvalue};

use std::fmt::{Debug, Formatter};

use byteorder::{BigEndian, ByteOrder};
use cesu8::from_java_cesu8;
pub use handle::*;
pub use raw::*;
pub use schema::*;
use std::collections::HashSet;
use std::convert::TryFrom;
pub use types::*;

mod ffi;
mod gc;
mod handle;
mod raw;
mod schema;
mod string;
mod types;

use crate::jvm::call::FlowControl;
pub use gc::*;

// TODO: Define shortcuts based on types
pub type ClassHandle = ObjectHandle;
pub type StringHandle = ObjectHandle;
pub type ObjArrayHandle = ObjectHandle;

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

pub enum ComputationalType {
    Category1,
    Category2,
}

pub trait StackValue: TryFrom<JavaValue, Error = FlowControl> + Into<JavaValue> {
    const CATEGORY: ComputationalType;
}

impl TryFrom<JavaValue> for jboolean {
    type Error = FlowControl;

    fn try_from(value: JavaValue) -> Result<Self, Self::Error> {
        match value {
            JavaValue::Byte(x) => Ok(x as jboolean),
            _ => Err(FlowControl::throw(
                "java/lang/UnsupportedOperationException",
            )),
        }
    }
}

impl TryFrom<JavaValue> for bool {
    type Error = FlowControl;

    fn try_from(value: JavaValue) -> Result<Self, Self::Error> {
        match value {
            JavaValue::Byte(x) => Ok(x != 0),
            _ => Err(FlowControl::throw(
                "java/lang/UnsupportedOperationException",
            )),
        }
    }
}

impl TryFrom<JavaValue> for jbyte {
    type Error = FlowControl;

    fn try_from(value: JavaValue) -> Result<Self, Self::Error> {
        match value {
            JavaValue::Byte(x) => Ok(x),
            JavaValue::Char(x) => Ok(x as jbyte),
            JavaValue::Short(x) => Ok(x as jbyte),
            JavaValue::Int(x) => Ok(x as jbyte),
            _ => Err(FlowControl::throw(
                "java/lang/UnsupportedOperationException",
            )),
        }
    }
}

impl TryFrom<JavaValue> for jchar {
    type Error = FlowControl;

    fn try_from(value: JavaValue) -> Result<Self, Self::Error> {
        match value {
            JavaValue::Char(x) => Ok(x),
            JavaValue::Byte(x) => Ok(x as jchar),
            JavaValue::Short(x) => Ok(x as jchar),
            JavaValue::Int(x) => Ok(x as jchar),
            _ => Err(FlowControl::throw(
                "java/lang/UnsupportedOperationException",
            )),
        }
    }
}

/// This version is dangerous since it does not preserve the value of the jchar and may produce an
/// exception if not in valid modified ceus8.
impl TryFrom<JavaValue> for char {
    type Error = FlowControl;

    fn try_from(value: JavaValue) -> Result<Self, Self::Error> {
        let character = match value {
            JavaValue::Char(x) => x,
            JavaValue::Byte(x) => x as jchar,
            JavaValue::Short(x) => x as jchar,
            JavaValue::Int(x) => x as jchar,
            _ => {
                return Err(FlowControl::throw(
                    "java/lang/UnsupportedOperationException",
                ))
            }
        };

        let mut buffer = [0u8; 2];
        BigEndian::write_u16(&mut buffer, character);

        match from_java_cesu8(&buffer) {
            Ok(string) => Ok(string.chars().next().unwrap()),
            Err(_) => Err(FlowControl::throw("java/lang/InternalError")),
        }
    }
}

impl TryFrom<JavaValue> for jshort {
    type Error = FlowControl;

    fn try_from(value: JavaValue) -> Result<Self, Self::Error> {
        match value {
            JavaValue::Byte(x) => Ok(x as jshort),
            JavaValue::Char(x) => Ok(x as jshort),
            JavaValue::Short(x) => Ok(x),
            JavaValue::Int(x) => Ok(x as jshort),
            _ => Err(FlowControl::throw(
                "java/lang/UnsupportedOperationException",
            )),
        }
    }
}

impl TryFrom<JavaValue> for jint {
    type Error = FlowControl;

    fn try_from(value: JavaValue) -> Result<Self, Self::Error> {
        match value {
            JavaValue::Byte(x) => Ok(x as jint),
            JavaValue::Char(x) => Ok(x as jint),
            JavaValue::Short(x) => Ok(x as jint),
            JavaValue::Int(x) => Ok(x),
            _ => Err(FlowControl::throw(
                "java/lang/UnsupportedOperationException",
            )),
        }
    }
}

impl TryFrom<JavaValue> for jfloat {
    type Error = FlowControl;

    fn try_from(value: JavaValue) -> Result<Self, Self::Error> {
        match value {
            JavaValue::Float(x) => Ok(x),
            _ => Err(FlowControl::throw(
                "java/lang/UnsupportedOperationException",
            )),
        }
    }
}

impl TryFrom<JavaValue> for jlong {
    type Error = FlowControl;

    fn try_from(value: JavaValue) -> Result<Self, Self::Error> {
        match value {
            JavaValue::Long(x) => Ok(x),
            _ => Err(FlowControl::throw(
                "java/lang/UnsupportedOperationException",
            )),
        }
    }
}

impl TryFrom<JavaValue> for jdouble {
    type Error = FlowControl;

    fn try_from(value: JavaValue) -> Result<Self, Self::Error> {
        match value {
            JavaValue::Double(x) => Ok(x),
            _ => Err(FlowControl::throw(
                "java/lang/UnsupportedOperationException",
            )),
        }
    }
}

impl TryFrom<JavaValue> for Option<ObjectHandle> {
    type Error = FlowControl;

    fn try_from(value: JavaValue) -> Result<Self, Self::Error> {
        match value {
            JavaValue::Reference(x) => Ok(x),
            _ => Err(FlowControl::throw(
                "java/lang/UnsupportedOperationException",
            )),
        }
    }
}

impl TryFrom<JavaValue> for ObjectHandle {
    type Error = FlowControl;

    fn try_from(value: JavaValue) -> Result<Self, Self::Error> {
        match value {
            JavaValue::Reference(Some(x)) => Ok(x),
            JavaValue::Reference(None) => Err(FlowControl::throw("java/lang/NullPointerException")),
            _ => Err(FlowControl::throw(
                "java/lang/UnsupportedOperationException",
            )),
        }
    }
}

impl From<jboolean> for JavaValue {
    fn from(x: jboolean) -> Self {
        JavaValue::Byte(x as jbyte)
    }
}

impl From<bool> for JavaValue {
    fn from(x: bool) -> Self {
        JavaValue::Byte(x as jbyte)
    }
}

impl From<jbyte> for JavaValue {
    fn from(x: jbyte) -> Self {
        JavaValue::Byte(x)
    }
}

impl From<jchar> for JavaValue {
    fn from(x: jchar) -> Self {
        JavaValue::Char(x)
    }
}

impl From<char> for JavaValue {
    fn from(x: char) -> Self {
        JavaValue::Char(x as jchar)
    }
}

impl From<jshort> for JavaValue {
    fn from(x: jshort) -> Self {
        JavaValue::Short(x)
    }
}

impl From<jint> for JavaValue {
    fn from(x: jint) -> Self {
        JavaValue::Int(x)
    }
}

impl From<jfloat> for JavaValue {
    fn from(x: jfloat) -> Self {
        JavaValue::Float(x)
    }
}

impl From<jlong> for JavaValue {
    fn from(x: jlong) -> Self {
        JavaValue::Long(x)
    }
}

impl From<jdouble> for JavaValue {
    fn from(x: jdouble) -> Self {
        JavaValue::Double(x)
    }
}

impl From<Option<ObjectHandle>> for JavaValue {
    fn from(x: Option<ObjectHandle>) -> Self {
        JavaValue::Reference(x)
    }
}

impl StackValue for jboolean {
    const CATEGORY: ComputationalType = ComputationalType::Category1;
}

impl StackValue for bool {
    const CATEGORY: ComputationalType = ComputationalType::Category1;
}

impl StackValue for jbyte {
    const CATEGORY: ComputationalType = ComputationalType::Category1;
}

impl StackValue for jchar {
    const CATEGORY: ComputationalType = ComputationalType::Category1;
}

impl StackValue for char {
    const CATEGORY: ComputationalType = ComputationalType::Category1;
}

impl StackValue for jshort {
    const CATEGORY: ComputationalType = ComputationalType::Category1;
}

impl StackValue for jint {
    const CATEGORY: ComputationalType = ComputationalType::Category1;
}

impl StackValue for jfloat {
    const CATEGORY: ComputationalType = ComputationalType::Category1;
}

impl StackValue for Option<ObjectHandle> {
    const CATEGORY: ComputationalType = ComputationalType::Category1;
}

impl StackValue for ObjectHandle {
    const CATEGORY: ComputationalType = ComputationalType::Category1;
}

impl StackValue for jlong {
    const CATEGORY: ComputationalType = ComputationalType::Category2;
}

impl StackValue for jdouble {
    const CATEGORY: ComputationalType = ComputationalType::Category2;
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
    pub fn expect_object(&self) -> ObjectHandle {
        match self {
            JavaValue::Reference(Some(x)) => *x,
            _ => panic!(),
        }
    }

    /// Helper function for conversion during match operations
    pub fn as_int(&self) -> Option<i64> {
        Some(match self {
            JavaValue::Byte(x) => *x as i64,
            JavaValue::Char(x) => *x as i64,
            JavaValue::Short(x) => *x as i64,
            JavaValue::Int(x) => *x as i64,
            JavaValue::Long(x) => *x,
            _ => return None,
        })
    }

    /// Helper function for conversion during match operations
    pub fn as_float(&self) -> Option<f64> {
        Some(match self {
            JavaValue::Byte(x) => *x as f64,
            JavaValue::Char(x) => *x as f64,
            JavaValue::Short(x) => *x as f64,
            JavaValue::Int(x) => *x as f64,
            JavaValue::Long(x) => *x as f64,
            JavaValue::Float(x) => *x as f64,
            JavaValue::Double(x) => *x,
            _ => return None,
        })
    }

    pub fn signum(&self) -> Option<i32> {
        Some(match self {
            JavaValue::Byte(x) => x.signum() as i32,
            JavaValue::Char(x) if *x == 0 => 0,
            JavaValue::Char(_) => 1,
            JavaValue::Short(x) => x.signum() as i32,
            JavaValue::Int(x) => x.signum(),
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

// FIXME: All int types can be compared and all comparisons are signed
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

impl Eq for JavaValue {}

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
            (JavaValue::Reference(Some(_)), JavaValue::Reference(None)) => Some(Ordering::Greater),
            (JavaValue::Reference(None), JavaValue::Reference(Some(_))) => Some(Ordering::Less),
            (JavaValue::Reference(None), JavaValue::Reference(None)) => Some(Ordering::Equal),
            (JavaValue::Reference(Some(a)), JavaValue::Reference(Some(b))) => {
                a.ptr().partial_cmp(&b.ptr())
            }
            (a, b) => match (a.as_int(), b.as_int()) {
                (Some(x), Some(y)) => x.partial_cmp(&y),
                _ => None,
            },
        }
    }
}

impl From<JavaValue> for jvalue {
    fn from(x: JavaValue) -> Self {
        match x {
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

impl NonCircularDebug for JavaValue {
    fn non_cyclical_fmt(
        &self,
        f: &mut Formatter<'_>,
        touched: &mut HashSet<ObjectHandle>,
    ) -> std::fmt::Result {
        match self {
            JavaValue::Byte(x) => write!(f, "{}u8", x),
            JavaValue::Char(x) => write!(f, "{:?}", std::char::from_u32(*x as u32).unwrap()),
            JavaValue::Short(x) => write!(f, "{}i16", x),
            JavaValue::Int(x) => write!(f, "{}i32", x),
            JavaValue::Float(x) => write!(f, "{}f32", x),
            JavaValue::Long(x) => write!(f, "{}i64", x),
            JavaValue::Double(x) => write!(f, "{}f64", x),
            JavaValue::Reference(x) => x.non_cyclical_fmt(f, touched),
        }
    }
}

macro_rules! non_circular_debug {
    ($type:ty: $fmt:expr) => {
        impl NonCircularDebug for $type {
            fn non_cyclical_fmt(
                &self,
                f: &mut Formatter<'_>,
                _: &mut HashSet<ObjectHandle>,
            ) -> std::fmt::Result {
                write!(f, $fmt, self)
            }
        }
    };
}

non_circular_debug! {u8: "{}u8"}
non_circular_debug! {i8: "{}i8"}
non_circular_debug! {i16: "{}i16"}
non_circular_debug! {i32: "{}i32"}
non_circular_debug! {i64: "{}i64"}
non_circular_debug! {f32: "{}f32"}
non_circular_debug! {f64: "{}f64"}

impl NonCircularDebug for jchar {
    fn non_cyclical_fmt(
        &self,
        f: &mut Formatter<'_>,
        _touched: &mut HashSet<ObjectHandle>,
    ) -> std::fmt::Result {
        write!(f, "{:?}", std::char::from_u32(*self as u32).unwrap())
    }
}

impl Debug for JavaValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut touched = HashSet::new();
        self.non_cyclical_fmt(f, &mut touched)
        // match self {
        //     JavaValue::Byte(x) => write!(f, "{}u8", x),
        //     JavaValue::Char(x) => write!(f, "{:?}", std::char::from_u32(*x as u32).unwrap()),
        //     JavaValue::Short(x) => write!(f, "{}i16", x),
        //     JavaValue::Int(x) => write!(f, "{}i32", x),
        //     JavaValue::Float(x) => write!(f, "{}f32", x),
        //     JavaValue::Reference(x) => write!(f, "{:?}", x),
        //     JavaValue::Long(x) => write!(f, "{}i64", x),
        //     JavaValue::Double(x) => write!(f, "{}f64", x),
        // }
    }
}

pub trait NonCircularDebug {
    fn non_cyclical_fmt(
        &self,
        f: &mut Formatter<'_>,
        touched: &mut HashSet<ObjectHandle>,
    ) -> std::fmt::Result;
}

// impl<T: Display> NonCircularDebug for T {
//     fn non_cyclical_fmt(&self, f: &mut Formatter<'_>, _: &mut HashSet<ObjectHandle>) -> std::fmt::Result {
//         self.fmt(f)
//     }
// }

impl<T: NonCircularDebug> NonCircularDebug for Vec<T> {
    fn non_cyclical_fmt(
        &self,
        f: &mut Formatter<'_>,
        touched: &mut HashSet<ObjectHandle>,
    ) -> std::fmt::Result {
        if self.is_empty() {
            return Ok(());
        }

        self[0].non_cyclical_fmt(f, touched)?;

        for item in &self[1..] {
            write!(f, ", ")?;
            item.non_cyclical_fmt(f, touched)?;
        }

        Ok(())
    }
}

// impl<T: NonCircularDebug> Debug for T {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let mut touched = HashSet::new();
//         self.non_cyclical_fmt(f, &mut touched)
//     }
// }
