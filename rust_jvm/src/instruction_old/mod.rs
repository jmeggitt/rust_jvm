#![allow(non_camel_case_types)]

use std::any::Any;
use std::fmt::Debug;
use std::io::{self, Cursor, Error, ErrorKind};
use std::ops::RangeInclusive;
use std::sync::Arc;

use byteorder::ReadBytesExt;
use parking_lot::RwLock;
use std::collections::HashMap;

pub use class::getstatic;

use array::*;
use class::*;
use cmp::*;
use convert::*;
use general::*;
use locals::*;
use math::*;
use push_const::*;
use stack::*;

use crate::jvm::call::{FlowControl, StackFrame};
use crate::jvm::JavaEnv;

#[macro_use]
mod macros;

mod array;
mod class;
mod cmp;
mod convert;
mod general;
mod locals;
mod math;
mod push_const;
mod stack;

pub mod instr {
    #[doc(inline)]
    pub use super::array::*;
    #[doc(inline)]
    pub use super::class::*;
    #[doc(inline)]
    pub use super::cmp::*;
    #[doc(inline)]
    pub use super::convert::*;
    #[doc(inline)]
    pub use super::general::*;
    #[doc(inline)]
    pub use super::locals::*;
    #[doc(inline)]
    pub use super::math::*;
    #[doc(inline)]
    pub use super::push_const::*;
    #[doc(inline)]
    pub use super::stack::*;
}

#[cfg(feature = "llvm")]
use llvm_sys::prelude::LLVMBuilderRef;

#[cfg(feature = "llvm")]
use crate::class::llvm::FunctionContext;

#[cfg(feature = "llvm")]
pub trait LLVMInstruction {
    unsafe fn add_impl(&self, _builder: LLVMBuilderRef, _cxt: &mut FunctionContext);
}

#[cfg(feature = "llvm")]
pub trait Instruction: Any + Debug + LLVMInstruction {
    fn write(&self, buffer: &mut Cursor<Vec<u8>>) -> io::Result<()>;

    fn exec(
        &self,
        _stack: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        panic!("Instruction not implemented for {:?}", self);
    }
}

#[cfg(not(feature = "llvm"))]
pub trait Instruction: Any + Debug {
    fn write(&self, buffer: &mut Cursor<Vec<u8>>) -> io::Result<()>;

    fn exec(
        &self,
        _stack: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        panic!("Instruction not implemented for {:?}", self);
    }
}

pub trait StaticInstruct: Instruction {
    const FORM: u8;
    const STRIDE: Option<RangeInclusive<u8>> = None;

    fn read(form: u8, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Box<dyn Instruction>>;
}

pub trait InstructionAction: Any {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl>;
}

#[derive(Default)]
pub struct InstructionReader {
    #[allow(clippy::type_complexity)]
    table: HashMap<u8, fn(u8, &mut Cursor<Vec<u8>>) -> io::Result<Box<dyn Instruction>>>,
}

impl InstructionReader {
    pub fn new() -> Self {
        let mut reader = InstructionReader::default();
        reader.do_register();
        reader
    }

    pub fn register<I: StaticInstruct>(&mut self) {
        if self.table.insert(I::FORM, I::read).is_some() {
            panic!("Got duplicate key: {}", I::FORM);
        }

        if let Some(range) = I::STRIDE {
            for alternate in range {
                if self.table.insert(alternate, I::read).is_some() {
                    panic!("Got duplicate key: {}", alternate);
                }
            }
        }
    }

    pub fn parse(
        &self,
        buffer: &mut Cursor<Vec<u8>>,
    ) -> io::Result<Vec<(u64, Box<dyn Instruction>)>> {
        let mut instructions = Vec::new();
        let len = buffer.get_ref().len();

        while (buffer.position() as usize) < len {
            let pos = buffer.position();
            let form = buffer.read_u8()?;

            match self.table.get(&form) {
                Some(reader) => instructions.push((pos, reader(form, buffer)?)),
                None => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Unknown instruction form: 0x{:X}", form),
                    ));
                }
            }
        }

        Ok(instructions)
    }

    fn do_register(&mut self) {
        self.register::<aaload>();
        self.register::<aastore>();
        self.register::<aconst_null>();
        self.register::<aload>();
        self.register::<anewarray>();
        self.register::<areturn>();
        self.register::<arraylength>();
        self.register::<astore>();
        self.register::<athrow>();
        self.register::<baload>();
        self.register::<bastore>();
        self.register::<bipush>();
        self.register::<caload>();
        self.register::<castore>();
        self.register::<checkcast>();
        self.register::<d2f>();
        self.register::<d2i>();
        self.register::<d2l>();
        self.register::<dadd>();
        self.register::<daload>();
        self.register::<dastore>();
        self.register::<dcmpg>();
        self.register::<dcmpl>();
        self.register::<dconst_0>();
        self.register::<dconst_1>();
        self.register::<ddiv>();
        self.register::<dload>();
        self.register::<dmul>();
        self.register::<dneg>();
        self.register::<drem>();
        self.register::<dreturn>();
        self.register::<dstore>();
        self.register::<dsub>();
        self.register::<dup>();
        self.register::<dup_x1>();
        self.register::<dup_x2>();
        self.register::<dup2>();
        self.register::<dup2_x1>();
        self.register::<dup2_x2>();
        self.register::<f2d>();
        self.register::<f2i>();
        self.register::<f2l>();
        self.register::<fadd>();
        self.register::<faload>();
        self.register::<fastore>();
        self.register::<fcmpg>();
        self.register::<fcmpl>();
        self.register::<fconst_0>();
        self.register::<fconst_1>();
        self.register::<fconst_2>();
        self.register::<fdiv>();
        self.register::<fload>();
        self.register::<fmul>();
        self.register::<fneg>();
        self.register::<frem>();
        self.register::<freturn>();
        self.register::<fstore>();
        self.register::<fsub>();
        self.register::<getfield>();
        self.register::<getstatic>();
        self.register::<goto>();
        self.register::<i2b>();
        self.register::<i2c>();
        self.register::<i2d>();
        self.register::<i2f>();
        self.register::<i2l>();
        self.register::<i2s>();
        self.register::<iadd>();
        self.register::<iaload>();
        self.register::<iand>();
        self.register::<iastore>();
        self.register::<iconst_m1>();
        self.register::<iconst_0>();
        self.register::<iconst_1>();
        self.register::<iconst_2>();
        self.register::<iconst_3>();
        self.register::<iconst_4>();
        self.register::<iconst_5>();
        self.register::<idiv>();
        self.register::<if_acmpeq>();
        self.register::<if_acmpne>();
        self.register::<if_icmpeq>();
        self.register::<if_icmpne>();
        self.register::<if_icmplt>();
        self.register::<if_icmpge>();
        self.register::<if_icmpgt>();
        self.register::<if_icmple>();
        self.register::<ifeq>();
        self.register::<ifne>();
        self.register::<iflt>();
        self.register::<ifge>();
        self.register::<ifgt>();
        self.register::<ifle>();
        self.register::<ifnonnull>();
        self.register::<ifnull>();
        self.register::<iload>();
        self.register::<imul>();
        self.register::<ineg>();
        self.register::<instanceof>();
        self.register::<invokespecial>();
        self.register::<invokestatic>();
        self.register::<invokevirtual>();
        self.register::<ior>();
        self.register::<irem>();
        self.register::<ireturn>();
        self.register::<ishl>();
        self.register::<ishr>();
        self.register::<istore>();
        self.register::<isub>();
        self.register::<iushr>();
        self.register::<ixor>();
        self.register::<jsr>();
        self.register::<l2d>();
        self.register::<l2f>();
        self.register::<l2i>();
        self.register::<ladd>();
        self.register::<laload>();
        self.register::<land>();
        self.register::<lastore>();
        self.register::<lcmp>();
        self.register::<lconst_0>();
        self.register::<lconst_1>();
        self.register::<ldc>();
        self.register::<ldc_w>();
        self.register::<ldc2_w>();
        self.register::<ldiv>();
        self.register::<lload>();
        self.register::<lmul>();
        self.register::<lneg>();
        self.register::<lor>();
        self.register::<lrem>();
        self.register::<lreturn>();
        self.register::<lshl>();
        self.register::<lshr>();
        self.register::<lstore>();
        self.register::<lsub>();
        self.register::<lushr>();
        self.register::<lxor>();
        self.register::<monitorenter>();
        self.register::<monitorexit>();
        self.register::<new>();
        self.register::<newarray>();
        self.register::<nop>();
        self.register::<pop>();
        self.register::<pop2>();
        self.register::<putfield>();
        self.register::<putstatic>();
        self.register::<ret>();
        self.register::<r#return>();
        self.register::<saload>();
        self.register::<sastore>();
        self.register::<sipush>();
        self.register::<swap>();

        self.register::<invokeinterface>();
        self.register::<iinc>();
        self.register::<lookupswitch>();
        self.register::<tableswitch>();
        self.register::<invokedynamic>();
        self.register::<goto_w>();
    }
}