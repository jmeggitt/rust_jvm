//! Used Instructions:
//!  - iload_N
//!  - aload_N
//!  - astore_N
//!  - invokespecial
//!  - getfield
//!  - putfield
//!  - return
//!  - getstatic
//!  - ldc
//!  - invokevirtual
//!  - new
//!  - dup
//!  - bipush
#![allow(non_camel_case_types)]

#[macro_use]
mod macros;

// import instructions
use general::*;
use class::*;
use push_const::*;
use stack::*;
use locals::*;


mod class;
mod load_store;
mod general;
mod push_const;
mod stack;
mod locals;

use crate::constant_pool::Constant;
use crate::jvm::{LocalVariable, JVM};
use byteorder::ReadBytesExt;
use hashbrown::HashMap;
use std::any::Any;
use std::fmt::Debug;
use std::io::{self, Cursor, Error, ErrorKind};
use std::ops::RangeInclusive;

pub trait Instruction: Any + Debug {
    fn write(&self, buffer: &mut Cursor<Vec<u8>>) -> io::Result<()>;

    fn exec(&self, _stack: &mut Vec<LocalVariable>, _pool: &[Constant], _jvm: &mut JVM) {
        panic!("Instruction not implemented for {:?}", self);
    }
}

pub trait StaticInstruct: Instruction {
    const FORM: u8;
    const STRIDE: Option<RangeInclusive<u8>> = None;

    fn read(form: u8, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Box<dyn Instruction>>;
}

pub trait InstructionAction: Any {
    fn exec(&self, stack: &mut Vec<LocalVariable>, pool: &[Constant], jvm: &mut JVM);
}

pub struct InstructionReader {
    table: HashMap<u8, fn(u8, &mut Cursor<Vec<u8>>) -> io::Result<Box<dyn Instruction>>>,
}

impl InstructionReader {
    pub fn new() -> Self {
        let mut reader = InstructionReader {
            table: HashMap::new(),
        };
        reader.do_register();
        reader
    }

    pub fn register<I: StaticInstruct>(&mut self) {
        if let Some(_) = self.table.insert(I::FORM, I::read) {
            panic!("Got duplicate key: {}", I::FORM);
        }

        if let Some(range) = I::STRIDE {
            for alternate in range {
                if let Some(_) = self.table.insert(alternate, I::read) {
                    panic!("Got duplicate key: {}", alternate);
                }
            }
        }
    }

    pub fn parse(&self, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Vec<Box<dyn Instruction>>> {
        let mut instructions = Vec::new();
        let len = buffer.get_ref().len() - 1;

        while (buffer.position() as usize) < len {
            let form = buffer.read_u8()?;

            match self.table.get(&form) {
                Some(reader) => instructions.push(reader(form, buffer)?),
                None => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Unknown instruction form: {:x}", form),
                    ))
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
    }
}

