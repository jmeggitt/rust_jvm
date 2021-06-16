use crate::class::BufferedRead;
use byteorder::ReadBytesExt;
use hashbrown::HashSet;
use std::io::{self, Cursor, Error, ErrorKind, Seek, SeekFrom};
use crate::jvm::LocalVariable;
use crate::jvm::bindings::jvalue;

#[derive(Debug, Clone, PartialEq)]
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
                FieldDescriptor::Object(_) | FieldDescriptor::Array(_) => panic!("Unable to read object from jvalue"),
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
