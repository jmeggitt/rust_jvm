use crate::jvm::call::StackFrame;
use crate::jvm::mem::JavaValue;

macro_rules! const_instruction {
    ($name:ident, $inst:literal, $($value:expr),+) => {
        pub fn $name(stack: &mut StackFrame) {
            $(stack.stack.push($value);)+
        }
    };
}

const_instruction! {aconst_null, 0x1, JavaValue::Reference(None)}
const_instruction! {dconst_0, 0xe, JavaValue::Double(0.0), JavaValue::Double(0.0)}
const_instruction! {dconst_1, 0xf, JavaValue::Double(1.0), JavaValue::Double(1.0)}
const_instruction! {fconst_0, 0xb, JavaValue::Float(0.0)}
const_instruction! {fconst_1, 0xc, JavaValue::Float(1.0)}
const_instruction! {fconst_2, 0xd, JavaValue::Float(2.0)}
const_instruction! {iconst_m1, 0x2, JavaValue::Int(-1)}
const_instruction! {iconst_0, 0x3, JavaValue::Int(0)}
const_instruction! {iconst_1, 0x4, JavaValue::Int(1)}
const_instruction! {iconst_2, 0x5, JavaValue::Int(2)}
const_instruction! {iconst_3, 0x6, JavaValue::Int(3)}
const_instruction! {iconst_4, 0x7, JavaValue::Int(4)}
const_instruction! {iconst_5, 0x8, JavaValue::Int(5)}
const_instruction! {lconst_0, 0x9, JavaValue::Long(0), JavaValue::Long(0)}
const_instruction! {lconst_1, 0xa, JavaValue::Long(1), JavaValue::Long(1)}
