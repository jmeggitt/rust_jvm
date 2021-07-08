use std::cell::{RefCell, UnsafeCell};
use std::io::{self, Cursor, Error, ErrorKind, Seek, SeekFrom};
use std::rc::Rc;

use byteorder::ReadBytesExt;
use hashbrown::HashSet;
use jni::sys::jvalue;

use crate::class::BufferedRead;
use crate::jvm::{LocalVariable, Object};
use libffi::middle::{Cif, Type};

#[derive(Debug, Clone, PartialEq, Hash)]
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
    pub fn to_string(&self) -> String {
        let mut string = String::new();

        match self {
            FieldDescriptor::Byte => string.push('B'),
            FieldDescriptor::Char => string.push('C'),
            FieldDescriptor::Double => string.push('D'),
            FieldDescriptor::Float => string.push('F'),
            FieldDescriptor::Int => string.push('I'),
            FieldDescriptor::Long => string.push('J'),
            FieldDescriptor::Short => string.push('S'),
            FieldDescriptor::Boolean => string.push('Z'),
            FieldDescriptor::Object(name) => {
                string.push('L');
                string.push_str(name);
                string.push(';');
            }
            FieldDescriptor::Array(entry) => {
                string.push('[');
                string.push_str(&entry.to_string());
            }
            FieldDescriptor::Void => string.push('V'),
            FieldDescriptor::Method { args, returns } => {
                string.push('(');

                for arg in args {
                    string.push_str(&arg.to_string());
                }

                string.push(')');
                string.push_str(&returns.to_string());
            }
        }

        string
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
                FieldDescriptor::Object(_) | FieldDescriptor::Array(_) => unsafe {
                    if value.l.is_null() {
                        LocalVariable::Reference(None)
                    } else {
                        debug!("Attempting to clone Rc through pointer!");
                        let reference = &*(value.l as *const Rc<UnsafeCell<Object>>);
                        let out = LocalVariable::Reference(Some(reference.clone()));
                        debug!("Got value {:?} from pointer", &out);
                        out
                    }
                },
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
        if let FieldDescriptor::Method {args, returns} = self {
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

