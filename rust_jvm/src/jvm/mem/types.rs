use std::io::{self, Error, ErrorKind, Read, Seek, SeekFrom};

use byteorder::ReadBytesExt;
use jni::sys::jvalue;
use std::collections::HashSet;

use crate::class::constant::ClassElement;
use crate::class::BufferedRead;
use crate::jvm::call::JavaEnvInvoke;
use crate::jvm::mem::{ConstTypeId, ManualInstanceReference, ObjectReference};
use crate::jvm::{JavaEnv, JavaValue, ObjectHandle};
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort};
use libffi::middle::{Cif, Type};
use parking_lot::RwLock;
use std::fmt::{Debug, Display, Formatter};
use std::ptr::null_mut;
use std::sync::Arc;

#[cfg(feature = "inkwell")]
use inkwell::{context::Context, types::PointerType};

pub trait JavaPrimitive: 'static + Sized + Copy + ConstTypeId {
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
        unsafe { ObjectHandle::from_ptr(val.l) }
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

impl FieldDescriptor {
    pub fn from_class(class: ObjectHandle) -> Self {
        if class.get_class() == "java/lang/Class" {
            let name = class.unwrap_as_class();
            return Self::from_class_name(&name);
        }

        assert_eq!(class.get_class(), "java/lang/invoke/MethodType");
        let instance = class.expect_instance();
        let instance_lock = instance.lock();
        let ptype: Option<ObjectHandle> = instance_lock.read_named_field("ptypes");
        let ptype = ptype.unwrap().expect_array::<Option<ObjectHandle>>();
        let ptype_lock = ptype.lock();
        let rtype: Option<ObjectHandle> = instance_lock.read_named_field("rtype");

        FieldDescriptor::Method {
            args: ptype_lock
                .iter()
                .copied()
                .map(Option::unwrap)
                .map(FieldDescriptor::from_class)
                .collect(),
            returns: Box::new(Self::from_class(rtype.unwrap())),
        }
    }

    pub fn from_class_name(name: &str) -> Self {
        match name {
            "boolean" => FieldDescriptor::Boolean,
            "byte" => FieldDescriptor::Byte,
            "char" => FieldDescriptor::Char,
            "short" => FieldDescriptor::Short,
            "int" => FieldDescriptor::Int,
            "long" => FieldDescriptor::Long,
            "float" => FieldDescriptor::Float,
            "double" => FieldDescriptor::Double,
            "void" => FieldDescriptor::Void,
            x => {
                if let Some(class_name) = x.strip_prefix('[') {
                    return FieldDescriptor::Array(Box::new(Self::from_class_name(class_name)));
                }

                FieldDescriptor::Object(x.to_string())
            }
        }
    }

    pub fn to_class(&self, jvm: &mut Arc<RwLock<JavaEnv>>) -> ObjectHandle {
        match self {
            FieldDescriptor::Byte => jvm.write().class_instance("byte"),
            FieldDescriptor::Char => jvm.write().class_instance("char"),
            FieldDescriptor::Double => jvm.write().class_instance("double"),
            FieldDescriptor::Float => jvm.write().class_instance("float"),
            FieldDescriptor::Int => jvm.write().class_instance("int"),
            FieldDescriptor::Long => jvm.write().class_instance("long"),
            FieldDescriptor::Short => jvm.write().class_instance("short"),
            FieldDescriptor::Boolean => jvm.write().class_instance("boolean"),
            FieldDescriptor::Object(name) => jvm.write().class_instance(name),
            FieldDescriptor::Array(boxed) => jvm.write().class_instance(&format!("[{}", boxed)),
            FieldDescriptor::Void => jvm.write().class_instance("void"),
            FieldDescriptor::Method { args, returns } => {
                let rtype = returns.to_class(jvm);
                let ptype = args
                    .iter()
                    .map(|x| Some(x.to_class(jvm)))
                    .collect::<Vec<Option<ObjectHandle>>>();
                let ptype = ObjectHandle::array_from_data(ptype);

                let schema = jvm.write().class_schema("java/lang/invoke/MethodType");
                let out = ObjectHandle::new(schema);
                jvm.invoke_virtual(
                    ClassElement::new(
                        "java/lang/invoke/MethodType",
                        "<init>",
                        "(Ljava/lang/Class;[Ljava/lang/Class;Z)V",
                    ),
                    out,
                    vec![
                        JavaValue::Reference(Some(rtype)),
                        JavaValue::Reference(Some(ptype)),
                        JavaValue::Byte(1),
                    ],
                )
                .unwrap();

                out
            }
        }
    }

    pub fn split_method(&self) -> Option<(&[FieldDescriptor], &FieldDescriptor)> {
        match self {
            FieldDescriptor::Method { args, returns } => Some((&args, &returns)),
            _ => None,
        }
    }
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
    pub fn word_len(args: &[FieldDescriptor]) -> usize {
        let mut size = 0;
        for arg in args {
            size += match arg {
                FieldDescriptor::Long | FieldDescriptor::Double => 2,
                _ => 1,
            };
        }

        size
    }

    pub fn assign_from(&self, value: JavaValue) -> Option<JavaValue> {
        Some(match (self, value) {
            (FieldDescriptor::Boolean, JavaValue::Byte(x)) => JavaValue::Byte((x != 0) as _),
            (FieldDescriptor::Boolean, JavaValue::Short(x)) => JavaValue::Byte((x != 0) as _),
            (FieldDescriptor::Boolean, JavaValue::Int(x)) => JavaValue::Byte((x != 0) as _),
            (FieldDescriptor::Boolean, JavaValue::Long(x)) => JavaValue::Byte((x != 0) as _),
            (FieldDescriptor::Byte, JavaValue::Byte(x)) => JavaValue::Byte(x as _),
            (FieldDescriptor::Short, JavaValue::Byte(x)) => JavaValue::Short(x as _),
            (FieldDescriptor::Short, JavaValue::Short(x)) => JavaValue::Short(x as _),
            (FieldDescriptor::Short, JavaValue::Char(x)) => JavaValue::Short(x as _),
            (FieldDescriptor::Short, JavaValue::Int(x)) => JavaValue::Short(x as _),
            (FieldDescriptor::Char, JavaValue::Byte(x)) => JavaValue::Char(x as _),
            (FieldDescriptor::Char, JavaValue::Short(x)) => JavaValue::Char(x as _),
            (FieldDescriptor::Char, JavaValue::Char(x)) => JavaValue::Char(x as _),
            (FieldDescriptor::Char, JavaValue::Int(x)) => JavaValue::Char(x as _),
            (FieldDescriptor::Int, JavaValue::Byte(x)) => JavaValue::Int(x as _),
            (FieldDescriptor::Int, JavaValue::Short(x)) => JavaValue::Int(x as _),
            (FieldDescriptor::Int, JavaValue::Char(x)) => JavaValue::Int(x as _),
            (FieldDescriptor::Int, JavaValue::Int(x)) => JavaValue::Int(x as _),
            (FieldDescriptor::Long, JavaValue::Byte(x)) => JavaValue::Long(x as _),
            (FieldDescriptor::Long, JavaValue::Short(x)) => JavaValue::Long(x as _),
            (FieldDescriptor::Long, JavaValue::Char(x)) => JavaValue::Long(x as _),
            (FieldDescriptor::Long, JavaValue::Int(x)) => JavaValue::Long(x as _),
            (FieldDescriptor::Long, JavaValue::Long(x)) => JavaValue::Long(x as _),
            (FieldDescriptor::Float, JavaValue::Float(x)) => JavaValue::Float(x as _),
            (FieldDescriptor::Double, JavaValue::Float(x)) => JavaValue::Double(x as _),
            (FieldDescriptor::Double, JavaValue::Double(x)) => JavaValue::Double(x as _),
            (FieldDescriptor::Object(_), JavaValue::Reference(x)) => JavaValue::Reference(x),
            (FieldDescriptor::Array(_), JavaValue::Reference(x)) => JavaValue::Reference(x),
            _ => return None,
        })
    }

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

    pub fn initial_local(&self) -> JavaValue {
        match self {
            Self::Byte => JavaValue::Byte(0),
            Self::Char => JavaValue::Char(0),
            Self::Double => JavaValue::Double(0.0),
            Self::Float => JavaValue::Float(0.0),
            Self::Int => JavaValue::Int(0),
            Self::Long => JavaValue::Long(0),
            Self::Short => JavaValue::Short(0),
            Self::Boolean => JavaValue::Byte(0),
            _ => JavaValue::Reference(None),
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self, FieldDescriptor::Object(_) | FieldDescriptor::Array(_))
    }

    // Clippy tried to suggest replacing this match with a massive single line matches! macro, but I
    // prefer the raw match.
    #[allow(clippy::match_like_matches_macro)]
    pub fn matches(&self, local: &JavaValue) -> bool {
        match (self, local) {
            (FieldDescriptor::Boolean, JavaValue::Byte(_)) => true,
            (FieldDescriptor::Boolean, JavaValue::Short(_)) => true,
            (FieldDescriptor::Boolean, JavaValue::Int(_)) => true,
            (FieldDescriptor::Boolean, JavaValue::Long(_)) => true,
            (FieldDescriptor::Byte, JavaValue::Byte(_)) => true,
            (FieldDescriptor::Char, JavaValue::Char(_)) => true,
            (FieldDescriptor::Short, JavaValue::Short(_)) => true,
            (FieldDescriptor::Int, JavaValue::Int(_)) => true,
            (FieldDescriptor::Float, JavaValue::Float(_)) => true,
            (FieldDescriptor::Long, JavaValue::Long(_)) => true,
            (FieldDescriptor::Double, JavaValue::Double(_)) => true,
            (FieldDescriptor::Object(_), JavaValue::Reference(_)) => true,
            (FieldDescriptor::Array(_), JavaValue::Reference(_)) => true,
            _ => false,
        }
    }

    pub fn cast(&self, value: jvalue) -> Option<JavaValue> {
        unsafe {
            Some(match self {
                FieldDescriptor::Byte => JavaValue::Byte(value.b),
                FieldDescriptor::Char => JavaValue::Char(value.c),
                FieldDescriptor::Double => JavaValue::Double(value.d),
                FieldDescriptor::Float => JavaValue::Float(value.f),
                FieldDescriptor::Int => JavaValue::Int(value.i),
                FieldDescriptor::Long => JavaValue::Long(value.j),
                FieldDescriptor::Short => JavaValue::Short(value.s),
                FieldDescriptor::Boolean => JavaValue::Byte(value.z as i8),
                FieldDescriptor::Object(_) | FieldDescriptor::Array(_) => {
                    JavaValue::Reference(ObjectHandle::from_ptr(value.l))
                    // if value.l.is_null() {
                    //     JavaValue::Reference(None)
                    // } else {
                    //     debug!("Attempting to clone Rc through pointer!");
                    //     let reference = &*(value.l as *const Rc<UnsafeCell<Object>>);
                    //     let out = JavaValue::Reference(Some(reference.clone()));
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
    fn read<T: Read + Seek>(buffer: &mut T) -> io::Result<Self> {
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
    // #[cfg(feature = "llvm")]
    // pub fn llvm_type(&self, context: &Context) {
    //     match self {
    //         FieldDescriptor::Byte => context.i8_type(),
    //         FieldDescriptor::Char => context.i16_type(),
    //         FieldDescriptor::Double => context.f64_type(),
    //         FieldDescriptor::Float => context.f32_type(),
    //         FieldDescriptor::Int => context.i32_type(),
    //         FieldDescriptor::Long => context.i64_type(),
    //         FieldDescriptor::Short => context.i16_type(),
    //         FieldDescriptor::Boolean => context.bool_type(),
    //         FieldDescriptor::Object(_) => PointerType::, None),
    //         FieldDescriptor::Array(_) => {}
    //         FieldDescriptor::Void => {}
    //         FieldDescriptor::Method { .. } => {}
    //     };
    // }

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
            let mut ffi_args = vec![Type::pointer(), Type::pointer()];
            ffi_args.extend(args.iter().map(Self::ffi_arg_type));
            let mut cif = Cif::new(ffi_args, returns.ffi_arg_type());

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
