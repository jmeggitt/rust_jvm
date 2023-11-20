use crate::jvm::call::StackFrame;
use crate::jvm::mem::JavaValue;

pub fn aload(frame: &mut StackFrame, index: u16) {
    let value = frame.locals[index as usize];
    assert!(matches!(&value, JavaValue::Reference(_)));
    frame.stack.push(value);
}

pub fn astore(frame: &mut StackFrame, index: u16) {
    let value = frame.stack.pop().unwrap();
    assert!(matches!(&value, JavaValue::Reference(_)));
    if matches!(
        frame.locals[index as usize],
        JavaValue::Long(_) | JavaValue::Double(_)
    ) {
        warn!(
            "Performed partial overwrite of type 2 computational type in slot {}",
            index
        );
        frame.locals[(index ^ 1) as usize] = JavaValue::Int(0);
    }

    frame.locals[index as usize] = value;
}

pub fn dload(frame: &mut StackFrame, index: u16) {
    frame.stack.push(frame.locals[index as usize]);
    frame.stack.push(frame.locals[index as usize + 1]);
}

pub fn dstore(frame: &mut StackFrame, index: u16) {
    frame.locals[index as usize + 1] = frame.stack.pop().unwrap();
    frame.locals[index as usize] = frame.stack.pop().unwrap();
}

pub fn fload(frame: &mut StackFrame, index: u16) {
    frame.stack.push(frame.locals[index as usize]);
}

pub fn fstore(frame: &mut StackFrame, index: u16) {
    if matches!(
        frame.locals[index as usize],
        JavaValue::Long(_) | JavaValue::Double(_)
    ) {
        warn!(
            "Performed partial overwrite of type 2 computational type in slot {}",
            index
        );
        frame.locals[(index ^ 1) as usize] = JavaValue::Int(0);
    }
    frame.locals[index as usize] = frame.stack.pop().unwrap();
}

pub fn iload(frame: &mut StackFrame, index: u16) {
    frame.stack.push(frame.locals[index as usize]);
}

pub fn istore(frame: &mut StackFrame, index: u16) {
    if matches!(
        frame.locals[index as usize],
        JavaValue::Long(_) | JavaValue::Double(_)
    ) {
        warn!(
            "Performed partial overwrite of type 2 computational type in slot {}",
            index
        );
        frame.locals[(index ^ 1) as usize] = JavaValue::Int(0);
    }
    frame.locals[index as usize] = frame.stack.pop().unwrap();
}

pub fn lload(frame: &mut StackFrame, index: u16) {
    frame.stack.push(frame.locals[index as usize]);
    frame.stack.push(frame.locals[index as usize + 1]);
}

pub fn lstore(frame: &mut StackFrame, index: u16) {
    frame.locals[index as usize + 1] = frame.stack.pop().unwrap();
    frame.locals[index as usize] = frame.stack.pop().unwrap();
}
