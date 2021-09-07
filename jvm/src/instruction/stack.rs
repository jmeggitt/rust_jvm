use crate::instruction::InstructionAction;
use crate::jvm::call::{FlowControl, StackFrame};
use crate::jvm::mem::JavaValue;
use crate::jvm::JavaEnv;

instruction! {@partial dup, 0x59}
instruction! {@partial dup_x1, 0x5a}
instruction! {@partial dup_x2, 0x5b}
instruction! {@partial dup2, 0x5c}
instruction! {@partial dup2_x1, 0x5d}
instruction! {@partial dup2_x2, 0x5e}

// FIXME: Most of these commands have multiple forms depending on the stack element length

impl InstructionAction for dup {
    fn exec(&self, frame: &mut StackFrame, _: &mut JavaEnv) -> Result<(), FlowControl> {
        frame.stack.push(frame.stack[frame.stack.len() - 1].clone());
        Ok(())
    }
}

impl InstructionAction for dup_x1 {
    fn exec(&self, frame: &mut StackFrame, _: &mut JavaEnv) -> Result<(), FlowControl> {
        let top = frame.stack.pop().unwrap();
        let top2 = frame.stack.pop().unwrap();

        frame.stack.push(top.clone());
        frame.stack.push(top2);
        frame.stack.push(top);
        Ok(())
    }
}

impl InstructionAction for dup_x2 {
    fn exec(&self, frame: &mut StackFrame, _: &mut JavaEnv) -> Result<(), FlowControl> {
        let top = frame.stack.pop().unwrap();
        let top2 = frame.stack.pop().unwrap();
        let top3 = frame.stack.pop().unwrap();

        frame.stack.push(top.clone());
        frame.stack.push(top3);
        frame.stack.push(top2);
        frame.stack.push(top);
        Ok(())
    }
}

impl InstructionAction for dup2 {
    fn exec(&self, frame: &mut StackFrame, _: &mut JavaEnv) -> Result<(), FlowControl> {
        if matches!(
            frame.stack[frame.stack.len() - 1],
            JavaValue::Long(_) | JavaValue::Double(_)
        ) {
            frame.stack.push(frame.stack[frame.stack.len() - 1].clone());
        } else {
            frame.stack.push(frame.stack[frame.stack.len() - 2].clone());
            frame.stack.push(frame.stack[frame.stack.len() - 2].clone());
        }

        Ok(())
    }
}

impl InstructionAction for dup2_x1 {
    fn exec(&self, frame: &mut StackFrame, _: &mut JavaEnv) -> Result<(), FlowControl> {
        frame.stack.insert(
            frame.stack.len() - 3,
            frame.stack[frame.stack.len() - 2].clone(),
        );
        frame.stack.insert(
            frame.stack.len() - 3,
            frame.stack[frame.stack.len() - 1].clone(),
        );
        Ok(())
    }
}

impl InstructionAction for dup2_x2 {
    fn exec(&self, frame: &mut StackFrame, _: &mut JavaEnv) -> Result<(), FlowControl> {
        frame.stack.insert(
            frame.stack.len() - 4,
            frame.stack[frame.stack.len() - 2].clone(),
        );
        frame.stack.insert(
            frame.stack.len() - 4,
            frame.stack[frame.stack.len() - 1].clone(),
        );
        Ok(())
    }
}

instruction! {@partial nop, 0x0}
instruction! {@partial pop, 0x57}
instruction! {@partial pop2, 0x58}
instruction! {@partial swap, 0x5f}

impl InstructionAction for nop {
    fn exec(&self, _frame: &mut StackFrame, _jvm: &mut JavaEnv) -> Result<(), FlowControl> {
        Ok(())
    }
}

impl InstructionAction for pop {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JavaEnv) -> Result<(), FlowControl> {
        frame.stack.pop().unwrap();
        Ok(())
    }
}

impl InstructionAction for pop2 {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JavaEnv) -> Result<(), FlowControl> {
        frame.stack.pop().unwrap();
        frame.stack.pop().unwrap();
        Ok(())
    }
}

impl InstructionAction for swap {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JavaEnv) -> Result<(), FlowControl> {
        let a = frame.stack.pop().unwrap();
        let b = frame.stack.pop().unwrap();

        frame.stack.push(a);
        frame.stack.push(b);
        Ok(())
    }
}
