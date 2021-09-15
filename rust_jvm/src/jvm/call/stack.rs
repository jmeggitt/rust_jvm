use crate::jvm::call::FlowControl;
use crate::jvm::mem::JavaValue;
use jni::sys::jlong;
use std::mem::size_of;

/// Mimics the functionality of a cpu for call invocations
pub struct VirtualMachine {
    operand_stack: Vec<JavaValue>,
    locals_table: Vec<JavaValue>,
    stack_ptr: usize,
    stack_base: usize,
    locals_base: usize,
    locals_top: usize,
}

impl VirtualMachine {
    /// Allocate the stack in memory immediately. Optionally, I could also more directly allocate
    /// the pages for basically no change in performance.
    pub fn new(stack_size: usize, locals_size: usize) -> Self {
        assert_eq!(stack_size % size_of::<u32>(), 0);
        assert_eq!(locals_size % size_of::<u32>(), 0);

        VirtualMachine {
            operand_stack: vec![JavaValue::Long(0); stack_size / size_of::<u32>()],
            locals_table: vec![JavaValue::Long(0); locals_size / size_of::<u32>()],
            stack_ptr: 0,
            stack_base: 0,
            locals_base: 0,
            locals_top: 0,
        }
    }

    pub fn pop(&mut self) -> JavaValue {
        assert!(self.stack_ptr > self.stack_base);
        self.stack_ptr -= 1;
        self.operand_stack[self.stack_ptr]
    }

    /// Pop multiple elements from the stack. Aimed towards method calls which need to pop their
    /// arguments.
    pub fn pop_group(&mut self, n: usize) -> Vec<JavaValue> {
        assert!(self.stack_ptr - n >= self.stack_base);
        let ret = Vec::from(&self.operand_stack[self.locals_top - n..self.locals_top]);
        self.stack_ptr -= n;
        ret
    }

    pub fn push(&mut self, val: JavaValue) -> Result<(), FlowControl> {
        if self.stack_ptr >= self.operand_stack.len() {
            return Err(FlowControl::error("java/lang/StackOverflowError"));
        }
        self.operand_stack[self.stack_ptr] = val;
        self.stack_ptr += 1;
        Ok(())
    }

    pub fn local(&self, index: usize) -> JavaValue {
        assert!(self.locals_base + index < self.locals_top);
        self.locals_table[self.locals_base + index]
    }

    pub fn set_local(&mut self, index: usize, value: JavaValue) {
        assert!(self.locals_base + index < self.locals_top);
        self.locals_table[self.locals_base + index] = value;
    }

    pub fn init_locals(&mut self, values: &[JavaValue]) {
        assert!(values.len() < self.locals_top - self.locals_base);
        self.locals_table[self.locals_base..self.locals_base + values.len()]
            .copy_from_slice(values);
    }

    pub fn init_frame(&mut self, locals: usize) -> Result<(), FlowControl> {
        assert!(self.locals_top + locals <= self.locals_table.len());
        self.push(JavaValue::Long(self.locals_base as jlong))?;
        self.locals_base = self.locals_top;
        self.locals_top += locals;

        self.push(JavaValue::Long(self.stack_base as jlong))?;
        self.stack_base = self.stack_ptr;
        Ok(())
    }

    pub fn drop_frame(&mut self) {
        assert!(self.stack_base > 0);
        self.stack_ptr = self.stack_base;

        // Reduce stack base to avoid violating frame bounds
        self.stack_base -= 1;
        self.stack_base = self.pop().as_int().unwrap() as usize;

        self.locals_top = self.locals_base;
        self.locals_base = self.pop().as_int().unwrap() as usize;
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        // Initialize with 8MB stack to match that of most linux machines
        // Local size is arbitrarily set to half the stack at 4MB
        VirtualMachine::new(8388608, 4194304)
    }
}
