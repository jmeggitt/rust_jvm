use crate::attribute::CodeAttribute;
use crate::constant_pool::Constant;
use crate::jvm::call::FlowControl;
use crate::jvm::mem::{JavaValue, ObjectHandle, ObjectReference};
use crate::jvm::JavaEnv;
use std::mem::replace;

pub struct StackFrame {
    // Comparable to the .text section of a binary
    pub constants: Vec<Constant>,
    // Values treated as registers
    pub locals: Vec<JavaValue>,
    // The stack frame
    pub stack: Vec<JavaValue>,
    // pub branch_offset: i64,
    // Work around so instructions can set the return value
    // pub returns: Option<Option<JavaValue>>,
    // Kinda like a sticky fault
    // pub throws: Option<JavaValue>,
}

impl StackFrame {
    pub fn new(
        max_locals: usize,
        max_stack: usize,
        constants: Vec<Constant>,
        mut args: Vec<JavaValue>,
    ) -> Self {
        if max_locals > args.len() {
            args.extend(vec![JavaValue::Long(0); max_locals - args.len()]);
        }

        StackFrame {
            constants,
            locals: args,
            stack: Vec::with_capacity(max_stack),
            // branch_offset: 0,
            // returns: None,
            // throws: None,
        }
    }

    pub fn debug_print(&self) {
        debug!("Stack Frame Debug:");
        // debug!("\tBranching Offset: {:?}", self.branch_offset);
        // debug!("\tReturn Slot: {:?}", &self.returns);

        debug!("\tLocal Variables: {}", self.locals.len());
        for (idx, local) in self.locals.iter().enumerate() {
            debug!("\t\t{}:\t{:?}", idx, local)
        }

        debug!(
            "\tOperand Stack: {}/{}",
            self.stack.len(),
            self.stack.capacity()
        );
        for (idx, local) in self.stack.iter().enumerate() {
            debug!("\t\t{}:\t{:?}", idx, local)
        }
    }

    pub fn exec(
        &mut self,
        jvm: &mut JavaEnv,
        code: &CodeAttribute,
    ) -> Result<Option<JavaValue>, FlowControl> {
        // let instructions = self.code.instructions.clone();
        for (offset, instruction) in &code.instructions {
            trace!("\t{}:\t{:?}", offset, instruction);
        }

        let mut rip = 0;
        loop {
            if rip >= code.instructions.len() {
                panic!("Reached function end without returning");
                // return Ok(None);
            }

            // let instruction = &self.code.instructions[self.rip];
            debug!("Executing instruction {:?}", &code.instructions[rip]);
            match code.instructions[rip].1.exec(self, jvm) {
                Err(FlowControl::Branch(mut branch_offset)) => {
                    while branch_offset != 0 {
                        let (current_pos, _) = code.instructions[rip];
                        rip = (rip as i64 + branch_offset.signum()) as usize;
                        branch_offset -= code.instructions[rip].0 as i64 - current_pos as i64;
                    }
                }
                Err(FlowControl::Throws(Some(e))) => {
                    let exception_class = e.get_class();

                    let position = code.instructions[rip].0;
                    match code.attempt_catch(position, &exception_class, &self.constants, jvm) {
                        Some(jump_dst) => {
                            debug!("Exception successfully caught, branching to catch block!");
                            let mut branch_offset = jump_dst as i64 - position as i64;

                            while branch_offset != 0 {
                                let (current_pos, _) = code.instructions[rip];
                                rip = (rip as i64 + branch_offset.signum()) as usize;
                                branch_offset -=
                                    code.instructions[rip].0 as i64 - current_pos as i64;
                            }
                        }
                        None => {
                            warn!("Exception not caught, Raising: {}", exception_class);
                            jvm.debug_print_call_stack();
                            return Err(FlowControl::Throws(Some(e)));
                        }
                    }
                }
                Err(x) => return Err(x),
                _ => rip += 1,
            };
        }
    }
}
