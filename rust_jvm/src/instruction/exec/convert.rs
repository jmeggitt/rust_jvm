use crate::jvm::call::StackFrame;
use crate::jvm::mem::{FieldDescriptor, JavaValue};

macro_rules! convert_instruction {
    ($name:ident, $from:ident -> $to:ident) => {
        pub fn $name(frame: &mut StackFrame) {
            convert_instruction!{@repeat $from, let _popped = frame.stack.pop().unwrap();}
            if let Some(JavaValue::$from(x)) = FieldDescriptor::$from.assign_from(_popped) {
                convert_instruction!{@repeat $to, frame.stack.push(JavaValue::$to(x as _));}
            } else {
                let name_str = stringify!($name);
                panic!("Could not perform {} for {:?}", name_str, _popped);
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
convert_instruction! {d2f, Double -> Float}
convert_instruction! {d2i, Double -> Int}
convert_instruction! {d2l, Double -> Long}
convert_instruction! {f2d, Float -> Double}
convert_instruction! {f2i, Float -> Int}
convert_instruction! {f2l, Float -> Long}
convert_instruction! {i2b, Int -> Byte}
convert_instruction! {i2c, Int -> Char}
convert_instruction! {i2d, Int -> Double}
convert_instruction! {i2f, Int -> Float}
convert_instruction! {i2l, Int -> Long}
convert_instruction! {i2s, Int -> Short}
convert_instruction! {l2d, Long -> Double}
convert_instruction! {l2f, Long -> Float}
convert_instruction! {l2i, Long -> Int}
