use crate::constant_pool::Constant;
use crate::instruction::InstructionAction;
use crate::jvm::{JVM, LocalVariable, StackFrame};

instruction! {@partial dup, 0x59}
instruction! {@partial dup_x1, 0x5a}
instruction! {@partial dup_x2, 0x5b}
instruction! {@partial dup2, 0x5c}
instruction! {@partial dup2_x1, 0x5d}
instruction! {@partial dup2_x2, 0x5e}

impl InstructionAction for dup {
    fn exec(&self, frame: &mut StackFrame, _: &mut JVM) {
        frame.stack.push(frame.stack[frame.stack.len() - 1].clone());
    }
}

impl InstructionAction for dup_x1 {
    fn exec(&self, frame: &mut StackFrame, _: &mut JVM) {
        let top = frame.stack.pop().unwrap();
        let top2 = frame.stack.pop().unwrap();

        frame.stack.push(top.clone());
        frame.stack.push(top2);
        frame.stack.push(top);
    }
}

impl InstructionAction for dup_x2 {
    fn exec(&self, frame: &mut StackFrame, _: &mut JVM) {
        let top = frame.stack.pop().unwrap();
        let top2 = frame.stack.pop().unwrap();
        let top3 = frame.stack.pop().unwrap();

        frame.stack.push(top.clone());
        frame.stack.push(top3);
        frame.stack.push(top2);
        frame.stack.push(top);
    }
}

impl InstructionAction for dup2 {
    fn exec(&self, frame: &mut StackFrame, _: &mut JVM) {
        frame.stack.push(frame.stack[frame.stack.len() - 2].clone());
        frame.stack.push(frame.stack[frame.stack.len() - 2].clone());
    }
}

impl InstructionAction for dup2_x1 {
    fn exec(&self, frame: &mut StackFrame, _: &mut JVM) {
        frame.stack.insert(
            frame.stack.len() - 3,
            frame.stack[frame.stack.len() - 2].clone(),
        );
        frame.stack.insert(
            frame.stack.len() - 3,
            frame.stack[frame.stack.len() - 1].clone(),
        );
    }
}

impl InstructionAction for dup2_x2 {
    fn exec(&self, frame: &mut StackFrame, _: &mut JVM) {
        frame.stack.insert(
            frame.stack.len() - 4,
            frame.stack[frame.stack.len() - 2].clone(),
        );
        frame.stack.insert(
            frame.stack.len() - 4,
            frame.stack[frame.stack.len() - 1].clone(),
        );
    }
}


instruction! {@partial nop, 0x0}
instruction! {@partial pop, 0x57}
instruction! {@partial pop2, 0x58}
instruction! {@partial swap, 0x5f}

impl InstructionAction for nop {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        // ez
    }
}


impl InstructionAction for pop {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        frame.stack.pop().unwrap();
    }
}


impl InstructionAction for pop2 {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        frame.stack.pop().unwrap();
        frame.stack.pop().unwrap();
    }
}


impl InstructionAction for swap {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        let a = frame.stack.pop().unwrap();
        let b = frame.stack.pop().unwrap();

        frame.stack.push(a);
        frame.stack.push(b);
    }
}

