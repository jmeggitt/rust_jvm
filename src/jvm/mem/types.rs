use std::cell::UnsafeCell;
use std::io::{self, Cursor, Error, ErrorKind, Seek, SeekFrom};
use std::rc::Rc;

use byteorder::ReadBytesExt;
use hashbrown::HashSet;
use jni::sys::jvalue;

use crate::class::BufferedRead;
use crate::jvm::{LocalVariable, ObjectHandle};
use libffi::middle::{Cif, Type};
use std::fmt::{Debug, Display, Formatter};
use std::ptr::{NonNull, null_mut};
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort};



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

impl JavaPrimitive for Option<ObjectHandle> {
    fn pack(self) -> jvalue {
        match self {
            Some(v) => jvalue { l: v.ptr() },
            None => jvalue { l: null_mut() },
        }
    }

    fn unpack(val: jvalue) -> Self {
        unsafe {
            ObjectHandle::from_ptr(val.l)
        }
    }

    fn descriptor() -> FieldDescriptor {
        FieldDescriptor::Object("java/lang/Object".to_string())
    }
}

#[derive(Clone, PartialEq, Hash)]
pub enum FieldDescriptor {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Boolean,
    Object(String),
    Array(Box<FieldDescriptor>),

    // Only accessible as part of a method descriptor return type
    Void,
    Method {
        args: Vec<FieldDescriptor>,
        returns: Box<FieldDescriptor>,
    },
}

impl Debug for FieldDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Display for FieldDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldDescriptor::Byte => write!(f, "B"),
            FieldDescriptor::Char => write!(f, "C"),
            FieldDescriptor::Double => write!(f, "D"),
            FieldDescriptor::Float => write!(f, "F"),
            FieldDescriptor::Int => write!(f, "I"),
            FieldDescriptor::Long => write!(f, "J"),
            FieldDescriptor::Short => write!(f, "S"),
            FieldDescriptor::Boolean => write!(f, "Z"),
            FieldDescriptor::Object(name) => write!(f, "L{};", name),
            FieldDescriptor::Array(entry) => write!(f, "[{}", entry),
            FieldDescriptor::Void => write!(f, "V"),
            FieldDescriptor::Method { args, returns } => {
                write!(f, "(")?;

                for arg in args {
                    write!(f, "{}", arg)?;
                }

                write!(f, "){}", returns)
            }
        }
    }
}

impl FieldDescriptor {
    pub fn class_usage(&self) -> HashSet<String> {
        let mut set = HashSet::new();
        match self {
            FieldDescriptor::Object(v) => {
                set.insert(v.to_string());
            }
            FieldDescriptor::Array(boxed) => set.extend(boxed.class_usage()),
            FieldDescriptor::Method { args, returns } => {
                args.iter().for_each(|x| set.extend(x.class_usage()));
                set.extend(returns.class_usage());
            }
            _ => {}
        }

        set
    }

    pub fn initial_local(&self) -> LocalVariable {
        match self {
            Self::Byte => LocalVariable::Byte(0),
            Self::Char => LocalVariable::Char(0),
            Self::Double => LocalVariable::Double(0.0),
            Self::Float => LocalVariable::Float(0.0),
            Self::Int => LocalVariable::Int(0),
            Self::Long => LocalVariable::Long(0),
            Self::Short => LocalVariable::Short(0),
            Self::Boolean => LocalVariable::Byte(0),
            _ => LocalVariable::Reference(None),
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self, FieldDescriptor::Object(_) | FieldDescriptor::Array(_))
    }

    // Clippy tried to suggest replacing this match with a massive single line matches! macro, but I
    // prefer the raw match.
    #[allow(clippy::match_like_matches_macro)]
    pub fn matches(&self, local: &LocalVariable) -> bool {
        match (self, local) {
            (FieldDescriptor::Byte, LocalVariable::Byte(_)) => true,
            (FieldDescriptor::Boolean, LocalVariable::Byte(_)) => true,
            (FieldDescriptor::Char, LocalVariable::Char(_)) => true,
            (FieldDescriptor::Short, LocalVariable::Short(_)) => true,
            (FieldDescriptor::Int, LocalVariable::Int(_)) => true,
            (FieldDescriptor::Float, LocalVariable::Float(_)) => true,
            (FieldDescriptor::Long, LocalVariable::Long(_)) => true,
            (FieldDescriptor::Double, LocalVariable::Double(_)) => true,
            (FieldDescriptor::Object(_), LocalVariable::Reference(_)) => true,
            (FieldDescriptor::Array(_), LocalVariable::Reference(_)) => true,
            _ => false,
        }
    }

    pub fn cast(&self, value: jvalue) -> Option<LocalVariable> {
        unsafe {
            Some(match self {
                FieldDescriptor::Byte => LocalVariable::Byte(value.b),
                FieldDescriptor::Char => LocalVariable::Char(value.c),
                FieldDescriptor::Double => LocalVariable::Double(value.d),
                FieldDescriptor::Float => LocalVariable::Float(value.f),
                FieldDescriptor::Int => LocalVariable::Int(value.i),
                FieldDescriptor::Long => LocalVariable::Long(value.j as i64),
                FieldDescriptor::Short => LocalVariable::Short(value.s),
                FieldDescriptor::Boolean => LocalVariable::Byte(value.z as i8),
                FieldDescriptor::Object(_) | FieldDescriptor::Array(_) => {
                    LocalVariable::Reference(ObjectHandle::from_ptr(value.l))
                    // if value.l.is_null() {
                    //     LocalVariable::Reference(None)
                    // } else {
                    //     debug!("Attempting to clone Rc through pointer!");
                    //     let reference = &*(value.l as *const Rc<UnsafeCell<Object>>);
                    //     let out = LocalVariable::Reference(Some(reference.clone()));
                    //     debug!("Got value {:?} from pointer", &out);
                    //     out
                    // }
                }
                _ => return None,
            })
        }
    }
}

impl BufferedRead for FieldDescriptor {
    fn read(buffer: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        Ok(match buffer.read_u8()? {
            b'B' => FieldDescriptor::Byte,
            b'C' => FieldDescriptor::Char,
            b'D' => FieldDescriptor::Double,
            b'F' => FieldDescriptor::Float,
            b'I' => FieldDescriptor::Int,
            b'J' => FieldDescriptor::Long,
            b'S' => FieldDescriptor::Short,
            b'Z' => FieldDescriptor::Boolean,
            b'V' => FieldDescriptor::Void,
            b'[' => FieldDescriptor::Array(Box::new(FieldDescriptor::read(buffer)?)),
            b'L' => {
                let mut arr = Vec::new();
                loop {
                    match buffer.read_u8()? {
                        b';' => break,
                        x => arr.push(x),
                    }
                }

                FieldDescriptor::Object(match String::from_utf8(arr) {
                    Ok(v) => v,
                    Err(e) => return Err(Error::new(ErrorKind::Other, e)),
                })
            }
            b'(' => {
                let mut args = Vec::new();
                loop {
                    match buffer.read_u8()? {
                        b')' => break,
                        _ => {
                            buffer.seek(SeekFrom::Current(-1))?;
                            args.push(FieldDescriptor::read(buffer)?);
                        }
                    }
                }

                FieldDescriptor::Method {
                    args,
                    returns: Box::new(FieldDescriptor::read(buffer)?),
                }
            }
            _ => return Err(Error::new(ErrorKind::Other, "malformed field descriptor")),
        })
    }
}

impl FieldDescriptor {
    pub fn ffi_arg_type(&self) -> Type {
        match self {
            FieldDescriptor::Byte => Type::i8(),
            FieldDescriptor::Char => Type::u16(),
            FieldDescriptor::Double => Type::f64(),
            FieldDescriptor::Float => Type::f32(),
            FieldDescriptor::Int => Type::i32(),
            FieldDescriptor::Long => Type::i64(),
            FieldDescriptor::Short => Type::i16(),
            FieldDescriptor::Boolean => Type::u8(),
            FieldDescriptor::Void => Type::void(),
            _ => Type::pointer(),
        }
    }

    pub fn build_cif(&self) -> Cif {
        if let FieldDescriptor::Method { args, returns } = self {
            let mut cif = Cif::new(args.iter().map(Self::ffi_arg_type), returns.ffi_arg_type());

            #[cfg(not(all(target_arch = "x86", windows)))]
                cif.set_abi(libffi::raw::ffi_abi_FFI_DEFAULT_ABI);

            // STDCALL is used on win32
            #[cfg(all(target_arch = "x86", windows))]
                cif.set_abi(libffi::raw::ffi_abi_FFI_STDCALL);

            cif
        } else {
            panic!("Attempted to construct Cif from non-call descriptor!")
        }
    }
}