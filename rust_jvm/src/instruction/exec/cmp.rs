use crate::jvm::call::{FlowControl, StackFrame};
use crate::jvm::mem::JavaValue;
use std::cmp::Ordering;

macro_rules! cmp_instruction {
    ($name:ident, $cond:expr) => {
        pub fn $name(frame: &mut StackFrame, jmp: i16) -> Result<(), FlowControl> {
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
                // debug!("Branching by {}", jmp);
                return Err(FlowControl::Branch(jmp as i64));
                // frame.branch_offset += jmp as i64;
            }
            Ok(())
        }
    };
}

cmp_instruction! {if_icmpeq, |x| x == Ordering::Equal}
cmp_instruction! {if_icmpne, |x| x != Ordering::Equal}
cmp_instruction! {if_icmplt, |x| x == Ordering::Less}
cmp_instruction! {if_icmpge, |x| x == Ordering::Equal || x == Ordering::Greater}
cmp_instruction! {if_icmpgt, |x| x == Ordering::Greater}
cmp_instruction! {if_icmple, |x| x == Ordering::Equal || x == Ordering::Less}
cmp_instruction! {if_acmpeq, |x| x == Ordering::Equal}
cmp_instruction! {if_acmpne, |x| x != Ordering::Equal}

macro_rules! cmp_zero_instruction {
    ($name:ident, $cond:expr) => {
        pub fn $name(frame: &mut StackFrame, jmp: i16) -> Result<(), FlowControl> {
            let val = frame.stack.pop().unwrap();
            assert!(matches!(
                &val,
                JavaValue::Byte(_) | JavaValue::Char(_) | JavaValue::Short(_) | JavaValue::Int(_)
            ));
            let order = val.signum().expect("Unable to get ordering for branching");

            if $cond(order) {
                // debug!("Branching by {}", jmp);
                // frame.branch_offset += jmp as i64;
                return Err(FlowControl::Branch(jmp as i64));
            }
            Ok(())
        }
    };
}

cmp_zero_instruction! {ifeq, |x| x == 0}
cmp_zero_instruction! {ifne, |x| x != 0}
cmp_zero_instruction! {iflt, |x| x == -1}
cmp_zero_instruction! {ifge, |x| x >= 0}
cmp_zero_instruction! {ifgt, |x| x == 1}
cmp_zero_instruction! {ifle, |x| x <= 0}

pub fn ifnonnull(frame: &mut StackFrame, jmp: i16) -> Result<(), FlowControl> {
    match frame.stack.pop() {
        Some(JavaValue::Reference(Some(_))) => Err(FlowControl::Branch(jmp as i64)),
        Some(JavaValue::Reference(_)) => Ok(()),
        x => {
            frame.debug_print();
            panic!("ifnonnull only accepts references: {:?}", x)
        }
    }
}

pub fn ifnull(frame: &mut StackFrame, jmp: i16) -> Result<(), FlowControl> {
    match frame.stack.pop() {
        Some(JavaValue::Reference(None)) => Err(FlowControl::Branch(jmp as i64)),
        Some(JavaValue::Reference(_)) => Ok(()),
        x => {
            frame.debug_print();
            panic!("ifnull only accepts references: {:?}", x)
        }
    }
}

pub fn fcmpg(frame: &mut StackFrame) {
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
}

pub fn fcmpl(frame: &mut StackFrame) {
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
}

pub fn dcmpg(frame: &mut StackFrame) {
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
}

pub fn dcmpl(frame: &mut StackFrame) {
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
}

pub fn lcmp(frame: &mut StackFrame) {
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
}
