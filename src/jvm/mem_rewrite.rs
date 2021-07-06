use std::alloc;
use std::alloc::Layout;
use std::cell::UnsafeCell;
use std::mem;
use std::ops::Deref;
use std::pin::Pin;
use std::ptr;
use std::rc::Rc;

use hashbrown::HashMap;
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort, jvalue};

pub trait JavaPrimitive {
    fn pack(self) -> jvalue;
    fn unpack(val: jvalue) -> Self;
}

macro_rules! define_primitive {
    ($name:ty: $ref:ident) => {
        impl JavaPrimitive for $name {
            fn pack(self) -> jvalue {
                jvalue {$ref: self}
            }

            fn unpack(val: jvalue) -> Self {
                unsafe {val.$ref}
            }
        }
    };
}

define_primitive!(jboolean: z);
define_primitive!(jbyte: b);
define_primitive!(jchar: c);
define_primitive!(jshort: s);
define_primitive!(jint: i);
define_primitive!(jlong: j);
define_primitive!(jfloat: f);
define_primitive!(jdouble: d);

// impl JavaPrimitive for jint {
//     fn pack(self) -> jvalue {
//         jvalue {i: self}
//     }
//
//     fn unpack(val: jvalue) -> Self {
//         unsafe {val.i}
//     }
// }


pub struct ObjectWrapper<T> {
    inner: Pin<Rc<T>>,
}

impl<T> ObjectWrapper<T> {
    pub fn new(val: T) -> Self {
        ObjectWrapper {
            inner: Rc::pin(val),
        }
    }

    pub fn into_raw(self) -> jobject {
        unsafe {
            Rc::into_raw(Pin::into_inner_unchecked(self.inner)) as jobject
        }
    }

    pub unsafe fn from_raw_unchecked(ptr: jobject) -> Self {
        ObjectWrapper {
            inner: Pin::new_unchecked(Rc::from_raw(ptr as _)),
        }
    }

    pub unsafe fn from_raw(ptr: jobject) -> Option<Self> {
        if ptr.is_null() {
            return None
        }

        Some(Self::from_raw_unchecked(ptr))
    }
}

// #[repr(C, packed)]
pub struct RawObject<T: ?Sized> {
    // schema: Rc<ClassSchema>,
    fields: UnsafeCell<T>,
}

pub enum ObjectType {
    Instance,
    Array,
}

impl RawObject<[u64]> {

    pub fn alloc(len: usize) {
        unsafe {
            // let offset = &(*(ptr::null::<Self>())).fields as *const _ as usize;

            // Find length of zero-element fields (probably mem::size_of::<usize>()).
            // let base_size = mem::size_of_val(&([0u64; 0] as [u64]));
            // let size = offset + base_size + len * mem::size_of::<u64>();

            // Align should be 0, but check just in case
            // let layout = alloc::Layout::from_size_align(size, mem::align_of::<Self>()).unwrap();


            let array = Layout::array::<u64>(len).unwrap();
            let layout = Layout::new::<RawObject<()>>().extend(array).unwrap().0;


            // let raw = alloc::alloc(layout);


        }
    }

    // pub fn readField

}

pub trait ObjectReference {
    fn get_class_schema(&self) -> ClassSchema;

    fn get_class(&self) -> String {
        self.get_class_schema().name.to_owned()
    }
}

pub trait InstanceReference: ObjectReference {
    fn write_field<T>(&self, offset: usize, val: T);
    fn read_field<T>(&self, offset: usize) -> T;
}

pub trait ArrayReference: ObjectReference {
    fn write_array<T>(&self, index: usize, val: T);
    fn read_array<T>(&self, index: usize) -> T;
    fn array_length<T>(&self) -> bool;
}

pub struct ClassSchema {
    name: String,
    super_class: Option<Rc<ClassSchema>>,
    field_offsets: HashMap<String, usize>,
}



