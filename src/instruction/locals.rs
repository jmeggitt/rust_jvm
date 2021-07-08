use crate::instruction::InstructionAction;
use crate::jvm::{StackFrame, JVM};

instruction! {@partial aload, 0x19, u8, 0x2a <-> 0x2d}
instruction! {@partial astore, 0x3a, u8, 0x4b <-> 0x4e}

instruction! {@partial fload, 0x17, u8, 0x22 <-> 0x25}
instruction! {@partial fstore, 0x38, u8, 0x43 <-> 0x46}

instruction! {@partial iload, 0x15, u8, 0x1a <-> 0x1d}
instruction! {@partial istore, 0x36, u8, 0x3b <-> 0x3e}

instruction! {@partial dload, 0x18, u8, 0x26 <-> 0x29}
instruction! {@partial dstore, 0x39, u8, 0x47 <-> 0x4a}

instruction! {@partial lload, 0x16, u8, 0x1e <-> 0x21}
instruction! {@partial lstore, 0x37, u8, 0x3f <->0x42}

impl InstructionAction for aload {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let aload(index) = *self;
        frame.stack.push(frame.locals[index as usize].clone());
    }
}

impl InstructionAction for astore {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let astore(index) = *self;
        frame.locals[index as usize] = frame.stack.pop().unwrap();
    }
}

impl InstructionAction for dload {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let dload(index) = *self;
        frame.stack.push(frame.locals[index as usize].clone());
    }
}

impl InstructionAction for dstore {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let dstore(index) = *self;
        frame.locals[index as usize] = frame.stack.pop().unwrap();
    }
}

impl InstructionAction for fload {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let fload(index) = *self;
        frame.stack.push(frame.locals[index as usize].clone());
    }
}

impl InstructionAction for fstore {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let fstore(index) = *self;
        frame.locals[index as usize] = frame.stack.pop().unwrap();
    }
}

impl InstructionAction for iload {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let iload(index) = *self;
        frame.stack.push(frame.locals[index as usize].clone());
    }
}

impl InstructionAction for istore {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let istore(index) = *self;
        frame.locals[index as usize] = frame.stack.pop().unwrap();
    }
}

impl InstructionAction for lload {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let lload(index) = *self;
        frame.stack.push(frame.locals[index as usize].clone());
    }
}

impl InstructionAction for lstore {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let lstore(index) = *self;
        frame.locals[index as usize] = frame.stack.pop().unwrap();
    }
}
