use crate::read::Readable;
use byteorder::ReadBytesExt;
use jni_sys::{jdouble, jfloat, jint, jlong};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::borrow::Cow;
use std::io;
use std::io::{Error, ErrorKind, Read};

pub enum Constant {
    Class(u16),
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    InterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    String(u16),
    Integer(jint),
    Float(jfloat),
    Long(jlong),
    Double(jdouble),
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    Utf8(String),
    // TODO
    MethodHandle {
        reference_kind: ReferenceKind,
        reference_index: u16,
    },
    MethodType(u16),
    Dynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    Module(u16),
    Package(u16),
}

impl Readable for Constant {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        Ok(match buffer.read_u8()? {
            1 => {
                let length = u16::read(buffer)?;
                let mut bytes = vec![0; length as usize];
                buffer.read_exact(&mut bytes)?;

                Self::Utf8(match cesu8::from_java_cesu8(&bytes) {
                    Ok(v) => v.to_string(),
                    Err(e) => return Err(Error::new(ErrorKind::InvalidInput, e)),
                })
            }
            3 => Self::Integer(jint::read(buffer)?),
            4 => Self::Float(jfloat::read(buffer)?),
            5 => Self::Long(jlong::read(buffer)?),
            6 => Self::Double(jdouble::read(buffer)?),
            7 => Self::Class(u16::read(buffer)?),
            8 => Self::String(u16::read(buffer)?),
            9 => Self::Double(u16::read(buffer)?),
            10 => Self::Double(u16::read(buffer)?),
            12 => Self::Double(u16::read(buffer)?),
            15 => Self::Double(u16::read(buffer)?),
            16 => Self::Double(u16::read(buffer)?),
            17 => Self::Double(u16::read(buffer)?),
            18 => Self::Double(u16::read(buffer)?),
            19 => Self::Double(u16::read(buffer)?),
            20 => Self::Double(u16::read(buffer)?),
            x => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("Invalid constant tag {}", x),
                ))
            }
        })
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive)]
pub enum ReferenceKind {
    GetField = 1,
    GetStatic = 2,
    PutField = 3,
    PutStatic = 4,
    InvokeVirtual = 5,
    InvokeStatic = 6,
    InvokeSpecial = 7,
    NewInvokeSpecial = 8,
    InvokeInterface = 9,
}

impl Readable for ReferenceKind {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        match Self::from_u8(buffer.read_u8()?) {
            Some(v) => Ok(v),
            None => Err(Error::new(
                ErrorKind::Other,
                "Reference kind value out of bounds!",
            )),
        }
    }
}

pub struct ConstantPool {}
