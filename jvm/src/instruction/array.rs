use crate::constant_pool::{Constant, ConstantClass};
use crate::instruction::InstructionAction;
use crate::jvm::mem::ArrayReference;
use crate::jvm::mem::{JavaValue, ObjectHandle};
use crate::jvm::JavaEnv;
use crate::jvm::call::{StackFrame, FlowControl};
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort};

macro_rules! array_instruction {
    (@$type:ident $name:ident, $inst:literal, $arr_type:ty, $local:ident) => {
        instruction! {@partial $name, $inst}
        impl InstructionAction for $name {
            fn exec(&self, frame: &mut StackFrame, _jvm: &mut JavaEnv) -> Result<(), FlowControl> {
                array_instruction! {@$type frame, $arr_type, $local}
            }
        }
    };
    (@load $frame:ident, $arr_type:ty, $local:ident) => {
        let index = $frame.stack.pop().unwrap().as_int().unwrap() as usize;
        if let JavaValue::Reference(Some(arr)) = $frame.stack.pop().unwrap() {
            let array = arr.expect_array::<$arr_type>();
            $frame
                .stack
                .push(JavaValue::$local(array.read_array(index) as _));
            //if let Object::Array { values, .. } = unsafe { &*arr.get() } {
            //    $frame.stack.push(values[index].clone());
            //}
        }
        Ok(())
    };
    (@store $frame:ident, $arr_type:ty, $local:ident) => {
        let value = $frame.stack.pop().unwrap();
        let index = $frame.stack.pop().unwrap().as_int().unwrap() as usize;

        if let JavaValue::Reference(Some(arr)) = $frame.stack.pop().unwrap() {
            if let JavaValue::$local(x) = value {
                let array = arr.expect_array::<$arr_type>();
                array.write_array(index, x as _);
                return Ok(());
            }
            // if let Object::Array { values, .. } = unsafe { &mut *arr.get() } {
            //     values[index] = value;
            //     return;
            // }
        }
        panic!("Attempted to store/load from non-array")
    };
}

array_instruction! {@load  aaload, 0x32, Option<ObjectHandle>, Reference}
array_instruction! {@store aastore, 0x53, Option<ObjectHandle>, Reference}
array_instruction! {@load  baload, 0x33, jbyte, Byte}
array_instruction! {@store bastore, 0x54, jbyte, Byte}
array_instruction! {@load  caload, 0x34, jchar, Char}
array_instruction! {@store castore, 0x55, jchar, Char}
array_instruction! {@load  daload, 0x31, jdouble, Double}
array_instruction! {@store dastore, 0x52, jdouble, Double}
array_instruction! {@load  faload, 0x30, jfloat, Float}
array_instruction! {@store fastore, 0x51, jfloat, Float}
array_instruction! {@load  iaload, 0x2e, jint, Int}
array_instruction! {@store iastore, 0x4f, jint, Int}
array_instruction! {@load  laload, 0x2f, jlong, Long}
array_instruction! {@store lastore, 0x50, jlong, Long}
array_instruction! {@load  saload, 0x35, jshort, Short}
array_instruction! {@store sastore, 0x56, jshort, Short}

instruction! {@partial anewarray, 0xbd, u16}

impl InstructionAction for anewarray {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JavaEnv) -> Result<(), FlowControl> {
        let anewarray(index) = *self;

        let class_name = match &frame.constants[index as usize - 1] {
            Constant::Class(ConstantClass { name_index }) => frame.constants
                [*name_index as usize - 1]
                .expect_utf8()
                .unwrap(),
            x => panic!("anewarray not implemented for {:?}", x),
        };

        debug!("Creating array for {}", &class_name);

        let length = match frame.stack.pop() {
            Some(JavaValue::Int(i)) => i,
            x => panic!("{:?} is not a valid array length", x),
        };

        // frame
        //     .stack
        //     .push(JavaValue::Reference(Some(Rc::new(UnsafeCell::new(
        //         Object::Array {
        //             values: vec![JavaValue::Reference(None); length as usize],
        //             element_type: FieldDescriptor::Object(class_name),
        //         },
        //     )))));
        frame
            .stack
            .push(JavaValue::Reference(Some(ObjectHandle::new_array::<Option<ObjectHandle>, >(length as usize))));
        Ok(())
    }
}

instruction! {@partial newarray, 0xbc, u8}

impl InstructionAction for newarray {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JavaEnv) -> Result<(), FlowControl> {
        let newarray(arr_type) = *self;

        let length = match frame.stack.pop() {
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

        // let (initial_value, type_name) = match arr_type {
        //     4 => (JavaValue::Byte(0), FieldDescriptor::Boolean),
        //     5 => (JavaValue::Char(0), FieldDescriptor::Char),
        //     6 => (JavaValue::Float(0.0), FieldDescriptor::Float),
        //     7 => (JavaValue::Double(0.0), FieldDescriptor::Double),
        //     8 => (JavaValue::Byte(0), FieldDescriptor::Byte),
        //     9 => (JavaValue::Short(0), FieldDescriptor::Short),
        //     10 => (JavaValue::Int(0), FieldDescriptor::Int),
        //     11 => (JavaValue::Long(0), FieldDescriptor::Long),
        //     x => panic!("Can not create array of type {}", x),
        // };

        // debug!("Creating array for {:?}", &type_name);

        // frame
        //     .stack
        //     .push(JavaValue::Reference(Some(Rc::new(UnsafeCell::new(
        //         Object::Array {
        //             values: vec![initial_value; length as usize],
        //             element_type: type_name,
        //         },
        //     )))));
        Ok(())
    }
}

instruction! {@partial arraylength, 0xbe}

impl InstructionAction for arraylength {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JavaEnv) -> Result<(), FlowControl> {
        if let Some(JavaValue::Reference(Some(val))) = frame.stack.pop() {
            match val.unknown_array_length() {
                Some(v) => frame.stack.push(JavaValue::Int(v as i32)),
                None => panic!("Attempted to get array length of non-array"),
            }
            // if let Object::Array { values, .. } = unsafe { &*val.get() } {
            //     frame.stack.push(JavaValue::Int(values.len() as i32));
            // }
        }
        Ok(())
    }
}