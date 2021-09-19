// use crate::constant_pool::{Constant, ConstantClass};
use crate::class::constant::{Constant, ConstantClass};
use crate::instruction::InstructionAction;
use crate::jvm::call::{FlowControl, StackFrame};
use crate::jvm::mem::ArrayReference;
use crate::jvm::mem::FieldDescriptor;
use crate::jvm::mem::{JavaValue, ObjectHandle};
use crate::jvm::JavaEnv;
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort};
use parking_lot::RwLock;
use std::sync::Arc;

macro_rules! array_instruction {
    (@$type:ident $name:ident, $inst:literal, $arr_type:ty, $local:ident, $($desc:expr)+) => {
        instruction! {@partial $name, $inst}
        impl InstructionAction for $name {
            fn exec(
                &self,
                frame: &mut StackFrame,
                _jvm: &mut std::sync::Arc<parking_lot::RwLock<crate::jvm::JavaEnv>>,
            ) -> Result<(), FlowControl> {
                array_instruction! {@$type frame, $arr_type, $local, $($desc)+}
            }
        }
    };
    (@load $frame:ident, $arr_type:ty, $local:ident, $($desc:expr)+) => {
        let index = $frame.stack.pop().unwrap().as_int().unwrap() as usize;
        if let JavaValue::Reference(Some(arr)) = $frame.stack.pop().unwrap() {
            let array = arr.expect_array::<$arr_type>();
            $($frame
                .stack
                .push(JavaValue::$local(array.read_array(index) as _));
            let _ = $desc;)+ // to get previous statement to repeat
        } else {
            panic!("xaload expected reference")
        }
        Ok(())
    };
    (@store $frame:ident, $arr_type:ty, $local:ident, $($desc:expr)+) => {
        $(let value = $desc.assign_from($frame.stack.pop().unwrap());)+
        let index = $frame.stack.pop().unwrap().as_int().unwrap() as usize;

        if let JavaValue::Reference(Some(arr)) = $frame.stack.pop().unwrap() {
            if let Some(JavaValue::$local(x)) = value {
                let array = arr.expect_array::<$arr_type>();
                array.write_array(index, x as _);
                return Ok(());
            }
        }
        panic!("Attempted to store/load from non-array: {:?}", value)
    };
}

array_instruction! {@load  aaload, 0x32, Option<ObjectHandle>, Reference, FieldDescriptor::Object(String::new())}
array_instruction! {@store aastore, 0x53, Option<ObjectHandle>, Reference, FieldDescriptor::Object(String::new())}
array_instruction! {@load  baload, 0x33, jbyte, Byte, FieldDescriptor::Byte}
array_instruction! {@store bastore, 0x54, jbyte, Byte, FieldDescriptor::Byte}
array_instruction! {@load  caload, 0x34, jchar, Char, FieldDescriptor::Char}
array_instruction! {@store castore, 0x55, jchar, Char, FieldDescriptor::Char}
array_instruction! {@load  daload, 0x31, jdouble, Double, FieldDescriptor::Double FieldDescriptor::Double}
array_instruction! {@store dastore, 0x52, jdouble, Double, FieldDescriptor::Double FieldDescriptor::Double}
array_instruction! {@load  faload, 0x30, jfloat, Float, FieldDescriptor::Float}
array_instruction! {@store fastore, 0x51, jfloat, Float, FieldDescriptor::Float}
array_instruction! {@load  iaload, 0x2e, jint, Int, FieldDescriptor::Int}
array_instruction! {@store iastore, 0x4f, jint, Int, FieldDescriptor::Int}
array_instruction! {@load  laload, 0x2f, jlong, Long, FieldDescriptor::Long FieldDescriptor::Long}
array_instruction! {@store lastore, 0x50, jlong, Long, FieldDescriptor::Long FieldDescriptor::Long}
array_instruction! {@load  saload, 0x35, jshort, Short, FieldDescriptor::Short}
array_instruction! {@store sastore, 0x56, jshort, Short, FieldDescriptor::Short}

instruction! {@partial anewarray, 0xbd, u16}

impl InstructionAction for anewarray {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let anewarray(index) = *self;

        let class_name = match &frame.constants[index as usize - 1] {
            Constant::Class(ConstantClass { name_index }) => frame.constants
                [*name_index as usize - 1]
                .expect_utf8()
                .unwrap(),
            x => panic!("anewarray not implemented for {:?}", x),
        };

        debug!("Creating array for {}", &class_name);

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
        Ok(())
    }
}

instruction! {@partial newarray, 0xbc, u8}

impl InstructionAction for newarray {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let newarray(arr_type) = *self;

        let length = match frame
            .stack
            .pop()
            .and_then(|x| FieldDescriptor::Int.assign_from(x))
        {
            Some(JavaValue::Int(i)) => i,
            x => panic!("{:?} is not a valid array length", x),
        } as usize;

        let handle = match arr_type {
            4 => ObjectHandle::new_array::<jboolean>(length),
            5 => ObjectHandle::new_array::<jchar>(length),
            6 => ObjectHandle::new_array::<jfloat>(length),
            7 => ObjectHandle::new_array::<jdouble>(length),
            8 => ObjectHandle::new_array::<jbyte>(length),
            9 => ObjectHandle::new_array::<jshort>(length),
            10 => ObjectHandle::new_array::<jint>(length),
            11 => ObjectHandle::new_array::<jlong>(length),
            x => panic!("Can not create array of type {}", x),
        };

        frame.stack.push(JavaValue::Reference(Some(handle)));
        Ok(())
    }
}

instruction! {@partial arraylength, 0xbe}

impl InstructionAction for arraylength {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        if let Some(JavaValue::Reference(Some(val))) = frame.stack.pop() {
            match val.unknown_array_length() {
                Some(v) => frame.stack.push(JavaValue::Int(v as i32)),
                None => panic!("Attempted to get array length of non-array"),
            }
        } else {
            panic!("Got null while attempting to read array length!")
        }
        Ok(())
    }
}
