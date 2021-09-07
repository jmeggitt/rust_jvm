use std::cmp::Ordering;

use crate::instruction::InstructionAction;
use crate::jvm::call::{FlowControl, StackFrame};
use crate::jvm::mem::JavaValue;
use crate::jvm::JavaEnv;

macro_rules! cmp_instruction {
    ($name:ident, $inst:literal, i16, $cond:expr) => {
        instruction! {@partial $name, $inst, i16}

        impl InstructionAction for $name {
            fn exec(&self, frame: &mut StackFrame, _: &mut JavaEnv) -> Result<(), FlowControl> {
                let Self(jmp) = *self;

                let val2 = frame.stack.pop().unwrap();
                let val1 = frame.stack.pop().unwrap();
                let order = match val1.partial_cmp(&val2) {
                    Some(v) => v,
                    None => panic!(
                        "Unable to get ordering for branching between {:?} and {:?}",
                        val1, val2
                    ),
                };
                //  .expect("Unable to get ordering for branching");

                if $cond(order) {
                    debug!("Branching by {}", jmp);
                    return Err(FlowControl::Branch(jmp as i64));
                    // frame.branch_offset += jmp as i64;
                }
                Ok(())
            }
        }
    };
}

macro_rules! cmp_zero_instruction {
    ($name:ident, $inst:literal, i16, $cond:expr) => {
        instruction! {@partial $name, $inst, i16}

        impl InstructionAction for $name {
            fn exec(&self, frame: &mut StackFrame, _: &mut JavaEnv) -> Result<(), FlowControl> {
                let Self(jmp) = *self;

                let val = frame.stack.pop().unwrap();
                let order = val.signum().expect("Unable to get ordering for branching");

                if $cond(order) {
                    debug!("Branching by {}", jmp);
                    // frame.branch_offset += jmp as i64;
                    return Err(FlowControl::Branch(jmp as i64));
                }
                Ok(())
            }
        }
    };
}

cmp_instruction! {if_icmpeq, 0x9f, i16, |x| x == Ordering::Equal}
cmp_instruction! {if_icmpne, 0xa0, i16, |x| x != Ordering::Equal}
cmp_instruction! {if_icmplt, 0xa1, i16, |x| x == Ordering::Less}
cmp_instruction! {if_icmpge, 0xa2, i16, |x| x == Ordering::Equal || x == Ordering::Greater}
cmp_instruction! {if_icmpgt, 0xa3, i16, |x| x == Ordering::Greater}
cmp_instruction! {if_icmple, 0xa4, i16, |x| x == Ordering::Equal || x == Ordering::Less}
cmp_instruction! {if_acmpeq, 0xa5, i16, |x| x == Ordering::Equal}
cmp_instruction! {if_acmpne, 0xa6, i16, |x| x != Ordering::Equal}

cmp_zero_instruction! {ifeq, 0x99, i16, |x| x == 0}
cmp_zero_instruction! {ifne, 0x9a, i16, |x| x != 0}
cmp_zero_instruction! {iflt, 0x9b, i16, |x| x == -1}
cmp_zero_instruction! {ifge, 0x9c, i16, |x| x >= 0}
cmp_zero_instruction! {ifgt, 0x9d, i16, |x| x == 1}
cmp_zero_instruction! {ifle, 0x9e, i16, |x| x <= 0}

instruction! {@partial ifnonnull, 0xc7, i16}
instruction! {@partial ifnull, 0xc6, i16}

impl InstructionAction for ifnonnull {
    fn exec(&self, frame: &mut StackFrame, _: &mut JavaEnv) -> Result<(), FlowControl> {
        let Self(jmp) = *self;

        if let Some(JavaValue::Reference(Some(_))) = frame.stack.pop() {
            debug!("Branching by {}", jmp);
            // frame.branch_offset += jmp as i64;
            return Err(FlowControl::Branch(jmp as i64));
        }
        Ok(())
    }
}

impl InstructionAction for ifnull {
    fn exec(&self, frame: &mut StackFrame, _: &mut JavaEnv) -> Result<(), FlowControl> {
        let Self(jmp) = *self;

        if let Some(JavaValue::Reference(None)) = frame.stack.pop() {
            debug!("Branching by {}", jmp);
            return Err(FlowControl::Branch(jmp as i64));
            // frame.branch_offset += jmp as i64;
        }
        Ok(())
    }
}
