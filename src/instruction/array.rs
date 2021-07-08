use std::cell::UnsafeCell;
use std::rc::Rc;

use crate::constant_pool::{Constant, ConstantClass};
use crate::instruction::InstructionAction;
use crate::jvm::{LocalVariable, Object, StackFrame, JVM};
use crate::types::FieldDescriptor;

macro_rules! array_instruction {
    (@$type:ident $name:ident, $inst:literal) => {
        instruction! {@partial $name, $inst}
        impl InstructionAction for $name {
            fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
                array_instruction! {@$type frame}
            }
        }
    };
    (@load $frame:ident) => {
        let index = $frame.stack.pop().unwrap().as_int().unwrap() as usize;
        if let LocalVariable::Reference(Some(arr)) = $frame.stack.pop().unwrap() {
            if let Object::Array { values, .. } = unsafe { &*arr.get() } {
                $frame.stack.push(values[index].clone());
            }
        }
    };
    (@store $frame:ident) => {
        let value = $frame.stack.pop().unwrap();
        let index = $frame.stack.pop().unwrap().as_int().unwrap() as usize;

        if let LocalVariable::Reference(Some(arr)) = $frame.stack.pop().unwrap() {
            if let Object::Array { values, .. } = unsafe { &mut *arr.get() } {
                values[index] = value;
                return;
            }
        }
        panic!("Attempted to store/load from non-array")
    };
}

array_instruction! {@load  aaload, 0x32}
array_instruction! {@store aastore, 0x53}
array_instruction! {@load  baload, 0x33}
array_instruction! {@store bastore, 0x54}
array_instruction! {@load  caload, 0x34}
array_instruction! {@store castore, 0x55}
array_instruction! {@load  daload, 0x31}
array_instruction! {@store dastore, 0x52}
array_instruction! {@load  faload, 0x30}
array_instruction! {@store fastore, 0x51}
array_instruction! {@load  iaload, 0x2e}
array_instruction! {@store iastore, 0x4f}
array_instruction! {@load  laload, 0x2f}
array_instruction! {@store lastore, 0x50}
array_instruction! {@load  saload, 0x35}
array_instruction! {@store sastore, 0x56}

instruction! {@partial anewarray, 0xbd, u16}

impl InstructionAction for anewarray {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
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
            Some(LocalVariable::Int(i)) => i,
            x => panic!("{:?} is not a valid array length", x),
        };

        frame
            .stack
            .push(LocalVariable::Reference(Some(Rc::new(UnsafeCell::new(
                Object::Array {
                    values: vec![LocalVariable::Reference(None); length as usize],
                    element_type: FieldDescriptor::Object(class_name),
                },
            )))));
    }
}

instruction! {@partial newarray, 0xbc, u8}

impl InstructionAction for newarray {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let newarray(arr_type) = *self;

        let (initial_value, type_name) = match arr_type {
            4 => (LocalVariable::Byte(0), FieldDescriptor::Boolean),
            5 => (LocalVariable::Char(0), FieldDescriptor::Char),
            6 => (LocalVariable::Float(0.0), FieldDescriptor::Float),
            7 => (LocalVariable::Double(0.0), FieldDescriptor::Double),
            8 => (LocalVariable::Byte(0), FieldDescriptor::Byte),
            9 => (LocalVariable::Short(0), FieldDescriptor::Short),
            10 => (LocalVariable::Int(0), FieldDescriptor::Int),
            11 => (LocalVariable::Long(0), FieldDescriptor::Long),
            x => panic!("Can not create array of type {}", x),
        };

        debug!("Creating array for {:?}", &type_name);

        let length = match frame.stack.pop() {
            Some(LocalVariable::Int(i)) => i,
            x => panic!("{:?} is not a valid array length", x),
        };

        frame
            .stack
            .push(LocalVariable::Reference(Some(Rc::new(UnsafeCell::new(
                Object::Array {
                    values: vec![initial_value; length as usize],
                    element_type: type_name,
                },
            )))));
    }
}

instruction! {@partial arraylength, 0xbe}

impl InstructionAction for arraylength {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        if let Some(LocalVariable::Reference(Some(val))) = frame.stack.pop() {
            if let Object::Array { values, .. } = unsafe { &*val.get() } {
                frame.stack.push(LocalVariable::Int(values.len() as i32));
            }
        }
    }
}
