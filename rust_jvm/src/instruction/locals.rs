use crate::instruction::InstructionAction;
use crate::jvm::call::{FlowControl, StackFrame};
use crate::jvm::mem::{FieldDescriptor, JavaValue};
use crate::jvm::JavaEnv;
use parking_lot::RwLock;
use std::sync::Arc;

#[cfg(feature = "llvm")]
use _llvm_imports::*;

#[cfg(feature = "llvm")]
mod _llvm_imports {
    pub use crate::class::llvm::{FunctionContext, LLVMInstruction};

    pub use llvm_sys::core::{LLVMBuildLoad, LLVMBuildStore};
    pub use llvm_sys::prelude::LLVMBuilderRef;
    pub use crate::c_str;
}


instruction! {aload, 0x19, u8, 0x2a <-> 0x2d}
instruction! {astore, 0x3a, u8, 0x4b <-> 0x4e}

instruction! {fload, 0x17, u8, 0x22 <-> 0x25}
instruction! {fstore, 0x38, u8, 0x43 <-> 0x46}

instruction! {iload, 0x15, u8, 0x1a <-> 0x1d}
instruction! {istore, 0x36, u8, 0x3b <-> 0x3e}

instruction! {dload, 0x18, u8, 0x26 <-> 0x29}
instruction! {dstore, 0x39, u8, 0x47 <-> 0x4a}

instruction! {lload, 0x16, u8, 0x1e <-> 0x21}
instruction! {lstore, 0x37, u8, 0x3f <->0x42}

impl InstructionAction for aload {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let aload(index) = *self;
        let value = frame.locals[index as usize];
        assert!(matches!(&value, JavaValue::Reference(_)));
        frame.stack.push(value);
        Ok(())
    }
}

#[cfg(feature = "llvm")]
impl LLVMInstruction for aload {
    unsafe fn add_impl(&self, builder: LLVMBuilderRef, cxt: &mut FunctionContext) {
        let aload(index) = *self;

        let operand_type = FieldDescriptor::Object("java/lang/Object".to_string());
        let local = cxt.get_operand_alloca(&operand_type, index as _);
        let value = LLVMBuildLoad(builder, local, c_str!("aload"));

        let destination = cxt.push_stack_alloca(&operand_type);

        LLVMBuildStore(builder, value, destination);
    }
}

impl InstructionAction for astore {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let astore(index) = *self;
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
        Ok(())
    }
}

impl InstructionAction for dload {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let dload(index) = *self;
        frame.stack.push(frame.locals[index as usize]);
        frame.stack.push(frame.locals[index as usize + 1]);
        Ok(())
    }
}

impl InstructionAction for dstore {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let dstore(index) = *self;
        frame.locals[index as usize + 1] = frame.stack.pop().unwrap();
        frame.locals[index as usize] = frame.stack.pop().unwrap();
        Ok(())
    }
}

impl InstructionAction for fload {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let fload(index) = *self;
        frame.stack.push(frame.locals[index as usize]);
        Ok(())
    }
}

impl InstructionAction for fstore {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let fstore(index) = *self;
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
        Ok(())
    }
}

impl InstructionAction for iload {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let iload(index) = *self;
        frame.stack.push(frame.locals[index as usize]);
        Ok(())
    }
}

impl InstructionAction for istore {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let istore(index) = *self;
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
        Ok(())
    }
}

impl InstructionAction for lload {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let lload(index) = *self;
        frame.stack.push(frame.locals[index as usize]);
        frame.stack.push(frame.locals[index as usize + 1]);
        Ok(())
    }
}

impl InstructionAction for lstore {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let lstore(index) = *self;
        frame.locals[index as usize + 1] = frame.stack.pop().unwrap();
        frame.locals[index as usize] = frame.stack.pop().unwrap();
        Ok(())
    }
}
