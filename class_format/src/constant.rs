use crate::read::Readable;
use byteorder::ReadBytesExt;
use jni_sys::{jdouble, jfloat, jint, jlong};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::borrow::{Borrow, Cow};
use std::fmt::{Debug, Formatter};
use std::io;
use std::io::{Error, ErrorKind, Read};
use std::ops::Index;

#[derive(Debug, Clone)]
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
    // FIXME: This would be more stable if it was left as a byte vec, but it ruins the formatting
    // Maybe create a new struct specifically for this type of formatting and conversion
    Utf8(String),
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
            9 => Self::FieldRef {
                class_index: u16::read(buffer)?,
                name_and_type_index: u16::read(buffer)?,
            },
            10 => Self::MethodRef {
                class_index: u16::read(buffer)?,
                name_and_type_index: u16::read(buffer)?,
            },
            11 => Self::InterfaceMethodRef {
                class_index: u16::read(buffer)?,
                name_and_type_index: u16::read(buffer)?,
            },
            12 => Self::NameAndType {
                name_index: u16::read(buffer)?,
                descriptor_index: u16::read(buffer)?,
            },
            15 => Self::MethodHandle {
                reference_kind: ReferenceKind::read(buffer)?,
                reference_index: u16::read(buffer)?,
            },
            16 => Self::MethodType(u16::read(buffer)?),
            17 => Self::Dynamic {
                bootstrap_method_attr_index: u16::read(buffer)?,
                name_and_type_index: u16::read(buffer)?,
            },
            18 => Self::InvokeDynamic {
                bootstrap_method_attr_index: u16::read(buffer)?,
                name_and_type_index: u16::read(buffer)?,
            },
            19 => Self::Module(u16::read(buffer)?),
            20 => Self::Package(u16::read(buffer)?),
            x => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("Invalid constant tag {}", x),
                ));
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

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct RawConstantPool {
    pool: Vec<Constant>,
}

impl Readable for RawConstantPool {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        let count = u16::read(buffer)?;
        let mut pool = Vec::with_capacity(count as usize);

        let mut index = 1;
        while index < count {
            let value = Constant::read(buffer)?;

            match &value {
                Constant::Long(..) | Constant::Double(..) => {
                    pool.push(value.clone());
                    pool.push(value);
                    index += 2;
                }
                _ => {
                    pool.push(value);
                    index += 1;
                }
            };
        }

        pool.shrink_to_fit();
        Ok(RawConstantPool { pool })
    }
}

impl Index<u16> for RawConstantPool {
    type Output = Constant;

    fn index(&self, index: u16) -> &Self::Output {
        &self.pool[index as usize - 1]
    }
}

// TODO: Maybe return results instead of panicking.
pub trait ConstantPool {
    fn text(&self, index: u16) -> &str;

    fn class_name(&self, index: u16) -> &str;

    fn name_and_type(&self, index: u16) -> (&str, &str);

    fn class_element_ref(&self, index: u16) -> (&str, &str, &str);

    fn class_element_desc(&self, index: u16) -> ClassElement {
        let (class, element, desc) = self.class_element_ref(index);
        ClassElement::new(class, element, desc)
    }
}

impl<T: Index<u16, Output = Constant>> ConstantPool for T {
    fn text(&self, index: u16) -> &str {
        match &self[index] {
            Constant::Utf8(text) => text.as_ref(),
            x => panic!("Expected Utf8 constant, but found {:?}", x),
        }
    }

    fn class_name(&self, index: u16) -> &str {
        match &self[index] {
            Constant::Class(name_index) => self.text(*name_index),
            x => panic!("Expected Class constant, but found {:?}", x),
        }
    }

    fn name_and_type(&self, index: u16) -> (&str, &str) {
        match &self[index] {
            Constant::NameAndType {
                name_index,
                descriptor_index,
            } => (self.text(*name_index), self.text(*descriptor_index)),
            x => panic!("Expected NameAndType constant, but found {:?}", x),
        }
    }

    fn class_element_ref(&self, index: u16) -> (&str, &str, &str) {
        let (class_index, name_and_type) = match &self[index] {
            Constant::FieldRef {
                class_index,
                name_and_type_index,
            }
            | Constant::MethodRef {
                class_index,
                name_and_type_index,
            }
            | Constant::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => (*class_index, *name_and_type_index),
            x => panic!(
                "Expected FieldRef/MethodRef/InterfaceMethodRef constant, but found {:?}",
                x
            ),
        };

        let (name, desc) = self.name_and_type(name_and_type);
        (self.class_name(class_index), name, desc)
    }
}

#[derive(Clone)]
pub struct ClassElement {
    pub class: String,
    pub element: String,
    pub desc: String,
}

impl ClassElement {
    pub fn new<S: ToString>(class: S, element: S, desc: S) -> Self {
        ClassElement {
            class: class.to_string(),
            element: element.to_string(),
            desc: desc.to_string(),
        }
    }

    // pub fn build_desc(&self) -> FieldDescriptor {
    //     match FieldDescriptor::read_str(&self.desc) {
    //         Ok(v) => v,
    //         Err(e) => panic!("Expected FieldDescriptor: {:?}", e),
    //     }
    // }
}

impl Debug for ClassElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{} {}", &self.class, &self.element, &self.desc)
    }
}
