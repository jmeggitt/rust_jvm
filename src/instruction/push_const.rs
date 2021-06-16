use crate::jvm::LocalVariable;

macro_rules! const_instruction {
    ($name:ident, $inst:literal, $value:expr) => {
        instruction! {@partial $name, $inst}

        impl crate::instruction::InstructionAction for $name {
            fn exec(&self, stack: &mut Vec<crate::jvm::LocalVariable>, _: &[crate::constant_pool::Constant], _: &mut crate::jvm::JVM) {
                stack.push($value);
            }
        }
    };
}

const_instruction! {aconst_null, 0x1, LocalVariable::Reference(None)}
const_instruction! {dconst_0, 0xe, LocalVariable::Double(0.0)}
const_instruction! {dconst_1, 0xf, LocalVariable::Double(1.0)}
const_instruction! {fconst_0, 0xb, LocalVariable::Float(0.0)}
const_instruction! {fconst_1, 0xc, LocalVariable::Float(1.0)}
const_instruction! {fconst_2, 0xd, LocalVariable::Float(2.0)}
const_instruction! {iconst_m1, 0x2, LocalVariable::Int(-1)}
const_instruction! {iconst_0, 0x3, LocalVariable::Int(0)}
const_instruction! {iconst_1, 0x4, LocalVariable::Int(1)}
const_instruction! {iconst_2, 0x5, LocalVariable::Int(2)}
const_instruction! {iconst_3, 0x6, LocalVariable::Int(3)}
const_instruction! {iconst_4, 0x7, LocalVariable::Int(4)}
const_instruction! {iconst_5, 0x8, LocalVariable::Int(5)}
const_instruction! {lconst_0, 0x9, LocalVariable::Long(0)}
const_instruction! {lconst_1, 0xa, LocalVariable::Long(1)}
