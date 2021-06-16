use crate::instruction::InstructionAction;
use crate::jvm::{JVM, LocalVariable};
use crate::constant_pool::Constant;


instruction! {@partial dup, 0x59}
instruction! {@partial dup_x1, 0x5a}
instruction! {@partial dup_x2, 0x5b}
instruction! {@partial dup2, 0x5c}
instruction! {@partial dup2_x1, 0x5d}
instruction! {@partial dup2_x2, 0x5e}


impl InstructionAction for dup {
    fn exec(&self, stack: &mut Vec<LocalVariable>, _: &[Constant], _: &mut JVM) {
        stack.push(stack[stack.len() - 1].clone());
    }
}


impl InstructionAction for dup_x1 {
    fn exec(&self, stack: &mut Vec<LocalVariable>, _: &[Constant], _: &mut JVM) {
        let top = stack.pop().unwrap();
        let top2 = stack.pop().unwrap();

        stack.push(top.clone());
        stack.push(top2);
        stack.push(top);
    }
}


impl InstructionAction for dup_x2 {
    fn exec(&self, stack: &mut Vec<LocalVariable>, _: &[Constant], _: &mut JVM) {
        let top = stack.pop().unwrap();
        let top2 = stack.pop().unwrap();
        let top3 = stack.pop().unwrap();

        stack.push(top.clone());
        stack.push(top3);
        stack.push(top2);
        stack.push(top);
    }
}

impl InstructionAction for dup2 {
    fn exec(&self, stack: &mut Vec<LocalVariable>, _: &[Constant], _: &mut JVM) {
        stack.push(stack[stack.len() - 2].clone());
        stack.push(stack[stack.len() - 2].clone());
    }
}


impl InstructionAction for dup2_x1 {
    fn exec(&self, stack: &mut Vec<LocalVariable>, _: &[Constant], _: &mut JVM) {
        stack.insert(stack.len() - 3, stack[stack.len() - 2].clone());
        stack.insert(stack.len() - 3, stack[stack.len() - 1].clone());
    }
}


impl InstructionAction for dup2_x2 {
    fn exec(&self, stack: &mut Vec<LocalVariable>, _: &[Constant], _: &mut JVM) {
        stack.insert(stack.len() - 4, stack[stack.len() - 2].clone());
        stack.insert(stack.len() - 4, stack[stack.len() - 1].clone());
    }
}

