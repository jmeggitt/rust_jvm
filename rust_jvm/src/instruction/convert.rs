use crate::instruction::InstructionAction;
use crate::jvm::call::{FlowControl, StackFrame};
use crate::jvm::mem::{FieldDescriptor, JavaValue};
use crate::jvm::JavaEnv;

use parking_lot::RwLock;
use std::sync::Arc;

macro_rules! convert_instruction {
    ($name:ident, $inst:literal, $from:ident -> $to:ident) => {
        instruction! {@partial $name, $inst}

        impl InstructionAction for $name {
            fn exec(
                &self,
                frame: &mut StackFrame,
                _jvm: &mut Arc<RwLock<JavaEnv>>,
            ) -> Result<(), FlowControl> {
                convert_instruction!{@repeat $from, let _popped = frame.stack.pop().unwrap();}
                if let Some(JavaValue::$from(x)) = FieldDescriptor::$from.assign_from(_popped) {
                    convert_instruction!{@repeat $to, frame.stack.push(JavaValue::$to(x as _));}
                    Ok(())
                } else {
                    panic!("Could not perform {:?} for {:?}", self, _popped);
                }
            }
        }
    };
    (@repeat Long, $($tokens:tt)+) => {
        $($tokens)+
        $($tokens)+
    };
    (@repeat Double, $($tokens:tt)+) => {
        $($tokens)+
        $($tokens)+
    };
    (@repeat Int, $($tokens:tt)+) => {$($tokens)+};
    (@repeat Float, $($tokens:tt)+) => {$($tokens)+};
    (@repeat Byte, $($tokens:tt)+) => {$($tokens)+};
    (@repeat Short, $($tokens:tt)+) => {$($tokens)+};
    (@repeat Char, $($tokens:tt)+) => {$($tokens)+};
}

// TODO: Incorrect results when converting computational types
convert_instruction! {d2f, 0x90, Double -> Float}
convert_instruction! {d2i, 0x8e, Double -> Int}
convert_instruction! {d2l, 0x8f, Double -> Long}
convert_instruction! {f2d, 0x8d, Float -> Double}
convert_instruction! {f2i, 0x8b, Float -> Int}
convert_instruction! {f2l, 0x8c, Float -> Long}
convert_instruction! {i2b, 0x91, Int -> Byte}
convert_instruction! {i2c, 0x92, Int -> Char}
convert_instruction! {i2d, 0x87, Int -> Double}
convert_instruction! {i2f, 0x86, Int -> Float}
convert_instruction! {i2l, 0x85, Int -> Long}
convert_instruction! {i2s, 0x93, Int -> Short}
convert_instruction! {l2d, 0x8a, Long -> Double}
convert_instruction! {l2f, 0x89, Long -> Float}
convert_instruction! {l2i, 0x88, Long -> Int}

// instruction! {@partial i2d, 0x87}
//
// impl InstructionAction for i2d {
// fn exec(
//     &self,
//     frame: &mut StackFrame,
//     _jvm: &mut Arc<RwLock<JavaEnv>>,
// ) -> Result<(), FlowControl> {
//     let popped = frame.stack.pop().unwrap();
//
//     if let Some(JavaValue::Int(x)) = FieldDescriptor::Int.assign_from(popped) {
//         let dst = x as _;
//
//     }
//
//     if let JavaValue::$from(x) = popped {
//         frame.stack.push(JavaValue::$to(x as _));
//         Ok(())
//     } else {
//         panic!("Could not perform {:?} for {:?}", self, popped);
//     }
// }
// }

// impl InstructionAction for d2f {
//     fn exec(&self, frame: &mut StackFrame, jvm: &mut JavaEnv) {
//         let popped = frame.stack.pop().unwrap();
//         if let JavaValue::Double(x) = popped {
//             frame.stack.push(JavaValue::Float(x as _));
//         } else {
//             panic!("Could not perform {:?} for {:?}", self, popped);
//         }
//     }
// }
