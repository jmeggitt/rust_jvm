//! I went back to verify that the instructions in this file adhere to the specification and add
//! extra checks. This file should be safe.

use crate::instruction::InstructionAction;
use crate::jvm::call::{FlowControl, StackFrame};
use crate::jvm::mem::JavaValue;
use crate::jvm::JavaEnv;
use parking_lot::RwLock;
use std::sync::Arc;

instruction! {@partial dup, 0x59}
instruction! {@partial dup_x1, 0x5a}
instruction! {@partial dup_x2, 0x5b}
instruction! {@partial dup2, 0x5c}
instruction! {@partial dup2_x1, 0x5d}
instruction! {@partial dup2_x2, 0x5e}

impl InstructionAction for dup {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let peeked = frame.stack[frame.stack.len() - 1];

        if matches!(&peeked, JavaValue::Long(_) | JavaValue::Double(_)) {
            panic!("dup can only be used on category 1 computational types")
        }

        frame.stack.push(peeked);
        Ok(())
    }
}

impl InstructionAction for dup_x1 {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let value1 = frame.stack.pop().unwrap();
        let value2 = frame.stack.pop().unwrap();

        if matches!(&value1, JavaValue::Long(_) | JavaValue::Double(_))
            || matches!(&value2, JavaValue::Long(_) | JavaValue::Double(_))
        {
            panic!("dup_x1 can only be used on category 1 computational types")
        }

        frame.stack.push(value1);
        frame.stack.push(value2);
        frame.stack.push(value1);
        Ok(())
    }
}

impl InstructionAction for dup_x2 {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let value1 = frame.stack.pop().unwrap();
        let value2 = frame.stack.pop().unwrap();
        let value3 = frame.stack.pop().unwrap();

        if matches!(&value1, JavaValue::Long(_) | JavaValue::Double(_)) {
            panic!("dup_x2 value1 can only be used on category 1 computational types")
        }

        // value2 and 3 must either be a single category 2 type or 2 category 1s
        assert_eq!(
            matches!(&value2, JavaValue::Long(_) | JavaValue::Double(_)),
            matches!(&value3, JavaValue::Long(_) | JavaValue::Double(_))
        );

        frame.stack.push(value1);
        frame.stack.push(value3);
        frame.stack.push(value2);
        frame.stack.push(value1);
        Ok(())
    }
}

impl InstructionAction for dup2 {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let value1 = frame.stack[frame.stack.len() - 1];
        let value2 = frame.stack[frame.stack.len() - 2];

        // value2 and 3 must either be a single category 2 type or 2 category 1s
        assert_eq!(
            matches!(&value1, JavaValue::Long(_) | JavaValue::Double(_)),
            matches!(&value2, JavaValue::Long(_) | JavaValue::Double(_))
        );

        frame.stack.push(value2);
        frame.stack.push(value1);
        Ok(())
    }
}

impl InstructionAction for dup2_x1 {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let value1 = frame.stack.pop().unwrap();
        let value2 = frame.stack.pop().unwrap();
        let value3 = frame.stack.pop().unwrap();

        if matches!(&value3, JavaValue::Long(_) | JavaValue::Double(_)) {
            panic!("dup2_x1 value3 can only be used on category 1 computational types")
        }

        // value2 and 3 must either be a single category 2 type or 2 category 1s
        assert_eq!(
            matches!(&value1, JavaValue::Long(_) | JavaValue::Double(_)),
            matches!(&value2, JavaValue::Long(_) | JavaValue::Double(_))
        );

        frame.stack.push(value2);
        frame.stack.push(value1);
        frame.stack.push(value3);
        frame.stack.push(value2);
        frame.stack.push(value1);
        Ok(())
    }
}

impl InstructionAction for dup2_x2 {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let value1 = frame.stack.pop().unwrap();
        let value2 = frame.stack.pop().unwrap();
        let value3 = frame.stack.pop().unwrap();
        let value4 = frame.stack.pop().unwrap();

        // value 1 and 2 must either be a single category 2 type or 2 category 1s
        assert_eq!(
            matches!(&value1, JavaValue::Long(_) | JavaValue::Double(_)),
            matches!(&value2, JavaValue::Long(_) | JavaValue::Double(_))
        );

        // value 3 and 4 must either be a single category 2 type or 2 category 1s
        assert_eq!(
            matches!(&value3, JavaValue::Long(_) | JavaValue::Double(_)),
            matches!(&value4, JavaValue::Long(_) | JavaValue::Double(_))
        );

        frame.stack.push(value2);
        frame.stack.push(value1);
        frame.stack.push(value4);
        frame.stack.push(value3);
        frame.stack.push(value2);
        frame.stack.push(value1);

        Ok(())
    }
}

instruction! {@partial nop, 0x0}
instruction! {@partial pop, 0x57}
instruction! {@partial pop2, 0x58}
instruction! {@partial swap, 0x5f}

impl InstructionAction for nop {
    fn exec(
        &self,
        _frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        Ok(())
    }
}

impl InstructionAction for pop {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        assert!(!matches!(
            frame.stack.pop().unwrap(),
            JavaValue::Long(_) | JavaValue::Double(_)
        ));
        Ok(())
    }
}

impl InstructionAction for pop2 {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        // value 1 and 2 must either be a single category 2 type or 2 category 1s
        assert_eq!(
            matches!(
                frame.stack.pop().unwrap(),
                JavaValue::Long(_) | JavaValue::Double(_)
            ),
            matches!(
                frame.stack.pop().unwrap(),
                JavaValue::Long(_) | JavaValue::Double(_)
            )
        );
        Ok(())
    }
}

impl InstructionAction for swap {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let value1 = frame.stack.pop().unwrap();
        let value2 = frame.stack.pop().unwrap();

        if matches!(&value1, JavaValue::Long(_) | JavaValue::Double(_))
            || matches!(&value2, JavaValue::Long(_) | JavaValue::Double(_))
        {
            panic!("swap can only be used on category 1 computational types")
        }

        frame.stack.push(value1);
        frame.stack.push(value2);
        Ok(())
    }
}
