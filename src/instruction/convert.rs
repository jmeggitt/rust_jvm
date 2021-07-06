use crate::instruction::InstructionAction;
use crate::jvm::{JVM, LocalVariable, StackFrame};

macro_rules! convert_instruction {
    ($name:ident, $inst:literal, $from:ident -> $to:ident) => {
        instruction! {@partial $name, $inst}

        impl InstructionAction for $name {
            fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
                let popped = frame.stack.pop().unwrap();
                if let LocalVariable::$from(x) = popped {
                    frame.stack.push(LocalVariable::$to(x as _));
                } else {
                    panic!("Could not perform {:?} for {:?}", self, popped);
                }
            }
        }
    };
}

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

// impl InstructionAction for d2f {
//     fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
//         let popped = frame.stack.pop().unwrap();
//         if let LocalVariable::Double(x) = popped {
//             frame.stack.push(LocalVariable::Float(x as _));
//         } else {
//             panic!("Could not perform {:?} for {:?}", self, popped);
//         }
//     }
// }