use crate::jvm::bindings::{jobject, _jobject, jvalue};
use std::rc::Rc;
use hashbrown::HashMap;
use std::os::raw::c_long;
use std::ptr::null_mut;
use crate::class::FieldInfo;
use crate::types::FieldDescriptor;

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

impl<'a> Into<jobject> for &'a Object {
    fn into(self) -> *mut _jobject {
        self as *const _ as *mut _jobject
    }
}


/// All types which may exist on the java stack frame
#[derive(Debug, Clone)]
pub enum LocalVariable {
    Byte(i8),
    Char(u16),
    Short(i16),
    Int(i32),
    Float(f32),
    Reference(Option<Rc<Object>>),
    // This is more of a token effort as I don't currently plan on using this value
    ReturnAddress(u32),

    // Double long values takes 2 spots and must be followed by padding
    Long(i64),
    Double(f64),
    Padding,
}

impl Into<Option<jvalue>> for LocalVariable {
    fn into(self) -> Option<jvalue> {
        unsafe {
            Some(match self {
                LocalVariable::Byte(x) => jvalue { b: x },
                LocalVariable::Char(x) => jvalue { c: x },
                LocalVariable::Short(x) => jvalue { s: x },
                LocalVariable::Int(x) => jvalue { i: x },
                LocalVariable::Float(x) => jvalue { f: x },
                LocalVariable::Reference(x) => jvalue {
                    l: match x {
                        Some(v) => &v.as_ref() as *const _ as *mut _jobject,
                        None => null_mut(),
                    }
                },
                LocalVariable::ReturnAddress(x) => panic!(),
                LocalVariable::Long(x) => jvalue { j: x as c_long },
                LocalVariable::Double(x) => jvalue { d: x },
                LocalVariable::Padding => return None,
            })
        }
    }
}

