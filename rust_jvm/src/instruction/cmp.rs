use std::cmp::Ordering;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::instruction::InstructionAction;
use crate::jvm::call::{FlowControl, StackFrame};
use crate::jvm::mem::JavaValue;
use crate::jvm::JavaEnv;

macro_rules! cmp_instruction {
    ($name:ident, $inst:literal, i16, $cond:expr) => {
        instruction! {@partial $name, $inst, i16}

        impl InstructionAction for $name {
            fn exec(
                &self,
                frame: &mut StackFrame,
                _: &mut Arc<RwLock<JavaEnv>>,
            ) -> Result<(), FlowControl> {
                let Self(jmp) = *self;

                let val2 = frame.stack.pop().unwrap();
                let val1 = frame.stack.pop().unwrap();

                // Only accept type 1 computational types
                assert!(!matches!(&val1, JavaValue::Long(_) | JavaValue::Double(_)));
                assert!(!matches!(&val2, JavaValue::Long(_) | JavaValue::Double(_)));
                // assert!(matches!(&val1, JavaValue::Byte(_) | JavaValue::Char(_) | JavaValue::Short(_) | JavaValue::Int(_)));
                // assert!(matches!(&val2, JavaValue::Byte(_) | JavaValue::Char(_) | JavaValue::Short(_) | JavaValue::Int(_)));

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
            fn exec(
                &self,
                frame: &mut StackFrame,
                _: &mut Arc<RwLock<JavaEnv>>,
            ) -> Result<(), FlowControl> {
                let Self(jmp) = *self;

                let val = frame.stack.pop().unwrap();
                assert!(matches!(
                    &val,
                    JavaValue::Byte(_)
                        | JavaValue::Char(_)
                        | JavaValue::Short(_)
                        | JavaValue::Int(_)
                ));
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
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let Self(jmp) = *self;

        match frame.stack.pop() {
            Some(JavaValue::Reference(Some(_))) => Err(FlowControl::Branch(jmp as i64)),
            Some(JavaValue::Reference(_)) => Ok(()),
            x => {
                frame.debug_print();
                panic!("ifnonnull only accepts references: {:?}", x)
            }
        }
    }
}

impl InstructionAction for ifnull {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let Self(jmp) = *self;

        match frame.stack.pop() {
            Some(JavaValue::Reference(None)) => Err(FlowControl::Branch(jmp as i64)),
            Some(JavaValue::Reference(_)) => Ok(()),
            x => {
                frame.debug_print();
                panic!("ifnull only accepts references: {:?}", x)
            }
        }
    }
}

instruction! {@partial fcmpg, 0x96}
instruction! {@partial fcmpl, 0x95}

impl InstructionAction for fcmpg {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let value2 = frame.stack.pop().unwrap();
        let value1 = frame.stack.pop().unwrap();

        if let (JavaValue::Float(val1), JavaValue::Float(val2)) = (value1, value2) {
            frame.stack.push(match val1.partial_cmp(&val2) {
                Some(Ordering::Less) => JavaValue::Int(-1),
                Some(Ordering::Equal) => JavaValue::Int(0),
                Some(Ordering::Greater) => JavaValue::Int(1),
                None => JavaValue::Int(1),
            });
        } else {
            panic!("fcmp requires 2 floats to operate!")
        }

        Ok(())
    }
}

impl InstructionAction for fcmpl {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let value2 = frame.stack.pop().unwrap();
        let value1 = frame.stack.pop().unwrap();

        if let (JavaValue::Float(val1), JavaValue::Float(val2)) = (value1, value2) {
            frame.stack.push(match val1.partial_cmp(&val2) {
                Some(Ordering::Less) => JavaValue::Int(-1),
                Some(Ordering::Equal) => JavaValue::Int(0),
                Some(Ordering::Greater) => JavaValue::Int(1),
                None => JavaValue::Int(-1),
            });
        } else {
            panic!("fcmp requires 2 floats to operate!")
        }

        Ok(())
    }
}

instruction! {@partial dcmpg, 0x98}
instruction! {@partial dcmpl, 0x97}

impl InstructionAction for dcmpg {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        frame.stack.pop().unwrap();
        let value2 = frame.stack.pop().unwrap();
        frame.stack.pop().unwrap();
        let value1 = frame.stack.pop().unwrap();

        if let (JavaValue::Double(val1), JavaValue::Double(val2)) = (value1, value2) {
            frame.stack.push(match val1.partial_cmp(&val2) {
                Some(Ordering::Less) => JavaValue::Int(-1),
                Some(Ordering::Equal) => JavaValue::Int(0),
                Some(Ordering::Greater) => JavaValue::Int(1),
                None => JavaValue::Int(1),
            });
        } else {
            panic!("dcmp requires 2 doubles to operate!")
        }

        Ok(())
    }
}

impl InstructionAction for dcmpl {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        frame.stack.pop().unwrap();
        let value2 = frame.stack.pop().unwrap();
        frame.stack.pop().unwrap();
        let value1 = frame.stack.pop().unwrap();

        if let (JavaValue::Double(val1), JavaValue::Double(val2)) = (value1, value2) {
            frame.stack.push(match val1.partial_cmp(&val2) {
                Some(Ordering::Less) => JavaValue::Int(-1),
                Some(Ordering::Equal) => JavaValue::Int(0),
                Some(Ordering::Greater) => JavaValue::Int(1),
                None => JavaValue::Int(-1),
            });
        } else {
            panic!("dcmp requires 2 doubles to operate!")
        }

        Ok(())
    }
}

instruction! {@partial lcmp, 0x94}

impl InstructionAction for lcmp {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        frame.stack.pop().unwrap();
        let value2 = frame.stack.pop().unwrap();
        frame.stack.pop().unwrap();
        let value1 = frame.stack.pop().unwrap();

        if let (JavaValue::Long(val1), JavaValue::Long(val2)) = (value1, value2) {
            frame.stack.push(match val1.cmp(&val2) {
                Ordering::Less => JavaValue::Int(-1),
                Ordering::Equal => JavaValue::Int(0),
                Ordering::Greater => JavaValue::Int(1),
            });
        } else {
            panic!("dcmp requires 2 doubles to operate!")
        }

        Ok(())
    }
}
