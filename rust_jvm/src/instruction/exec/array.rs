use crate::instruction::{ClassConstIndex, PrimitiveType};
use crate::jvm::call::StackFrame;
use crate::jvm::mem::{ArrayReference, FieldDescriptor, JavaValue, ObjectHandle};
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort};

macro_rules! impl_load_instruction {
    ($name:ident, $arr_type:ty, $local:ident, $($desc:expr)+) => {
        pub fn $name(frame: &mut StackFrame) {
            let index = frame.stack.pop().unwrap().as_int().unwrap() as usize;
            if let JavaValue::Reference(Some(arr)) = frame.stack.pop().unwrap() {
                let array = arr.expect_array::<$arr_type>();
                let lock = array.lock();
                $(
                    frame.stack.push(JavaValue::$local(lock.read_array(index) as _));
                    let _ = $desc; // to get previous statement to repeat
                )+
            } else {
                panic!("xaload expected reference")
            }
        }
    };
}

impl_load_instruction!(
    aaload,
    Option<ObjectHandle>,
    Reference,
    FieldDescriptor::Object(String::new())
);
impl_load_instruction!(baload, jbyte, Byte, FieldDescriptor::Byte);
impl_load_instruction!(caload, jchar, Char, FieldDescriptor::Char);
impl_load_instruction! (daload, jdouble, Double, FieldDescriptor::Double FieldDescriptor::Double);
impl_load_instruction!(faload, jfloat, Float, FieldDescriptor::Float);
impl_load_instruction!(iaload, jint, Int, FieldDescriptor::Int);
impl_load_instruction! (laload, jlong, Long, FieldDescriptor::Long FieldDescriptor::Long);
impl_load_instruction!(saload, jshort, Short, FieldDescriptor::Short);

macro_rules! impl_store_instruction {
    ($name:ident, $arr_type:ty, $local:ident, $($desc:expr)+) => {
        pub fn $name(frame: &mut StackFrame) {
            $(let _value = $desc.assign_from(frame.stack.pop().unwrap());)+
            let index = frame.stack.pop().unwrap().as_int().unwrap() as usize;

            if let JavaValue::Reference(Some(arr)) = frame.stack.pop().unwrap() {
                if let Some(JavaValue::$local(x)) = _value {
                    let array = arr.expect_array::<$arr_type>();
                    let mut lock = array.lock();
                    lock.write_array(index, x as _);
                    return;
                }
            }
            panic!("Attempted to store/load from non-array: {:?}", _value)
        }
    };
}

impl_store_instruction!(
    aastore,
    Option<ObjectHandle>,
    Reference,
    FieldDescriptor::Object(String::new())
);
impl_store_instruction!(bastore, jbyte, Byte, FieldDescriptor::Byte);
impl_store_instruction!(castore, jchar, Char, FieldDescriptor::Char);
impl_store_instruction! (dastore, jdouble, Double, FieldDescriptor::Double FieldDescriptor::Double);
impl_store_instruction!(fastore, jfloat, Float, FieldDescriptor::Float);
impl_store_instruction!(iastore, jint, Int, FieldDescriptor::Int);
impl_store_instruction! (lastore, jlong, Long, FieldDescriptor::Long FieldDescriptor::Long);
impl_store_instruction!(sastore, jshort, Short, FieldDescriptor::Short);

pub fn anewarray(frame: &mut StackFrame, index: ClassConstIndex) {
    let class_name = frame.constants.class_name(index);

    trace!("Creating array for {}", &class_name);

    let length = match frame
        .stack
        .pop()
        .and_then(|x| FieldDescriptor::Int.assign_from(x))
    {
        Some(JavaValue::Int(i)) => i,
        x => panic!("{:?} is not a valid array length", x),
    };

    frame
        .stack
        .push(JavaValue::Reference(Some(ObjectHandle::new_array::<
            Option<ObjectHandle>,
        >(length as usize))));
}

pub fn newarray(frame: &mut StackFrame, array_type: PrimitiveType) {
    let length = match frame
        .stack
        .pop()
        .and_then(|x| FieldDescriptor::Int.assign_from(x))
    {
        Some(JavaValue::Int(i)) => i,
        x => panic!("{:?} is not a valid array length", x),
    } as usize;

    let handle = match array_type {
        PrimitiveType::Boolean => ObjectHandle::new_array::<jboolean>(length),
        PrimitiveType::Char => ObjectHandle::new_array::<jchar>(length),
        PrimitiveType::Float => ObjectHandle::new_array::<jfloat>(length),
        PrimitiveType::Double => ObjectHandle::new_array::<jdouble>(length),
        PrimitiveType::Byte => ObjectHandle::new_array::<jbyte>(length),
        PrimitiveType::Short => ObjectHandle::new_array::<jshort>(length),
        PrimitiveType::Int => ObjectHandle::new_array::<jint>(length),
        PrimitiveType::Long => ObjectHandle::new_array::<jlong>(length),
    };

    frame.stack.push(JavaValue::Reference(Some(handle)));
}

pub fn arraylength(frame: &mut StackFrame) {
    if let Some(JavaValue::Reference(Some(val))) = frame.stack.pop() {
        match val.unknown_array_length() {
            Some(v) => frame.stack.push(JavaValue::Int(v as i32)),
            None => panic!("Attempted to get array length of non-array"),
        }
    } else {
        panic!("Got null while attempting to read array length!")
    }
}
