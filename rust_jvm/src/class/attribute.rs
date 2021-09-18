use std::io;
use std::io::{Cursor, Read, Seek};

use byteorder::{BigEndian, ReadBytesExt};

use crate::class::class_file::AttributeInfo;
use crate::class::constant::Constant;
use crate::class::BufferedRead;
use crate::instruction::Instruction;
use crate::instruction::InstructionReader;
use crate::jvm::JavaEnv;

#[derive(Debug)]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub instructions: Vec<(u64, Box<dyn Instruction>)>,
    pub exception_table: Vec<ExceptionRange>,
    pub attributes: Vec<AttributeInfo>,
}

impl CodeAttribute {
    pub fn attempt_catch(
        &self,
        pos: u64,
        class: &str,
        pool: &[Constant],
        jvm: &mut JavaEnv,
    ) -> Option<u64> {
        // I assume that the first one that fits is the one to use?
        for range in self.exception_table.iter().copied() {
            if pos < range.try_start as u64 || pos > range.try_end as u64 {
                continue;
            }

            let index = pool[range.catch_type as usize - 1].expect_class().unwrap();
            let catch_target = pool[index as usize - 1].expect_utf8().unwrap();

            if jvm.instanceof(class, &catch_target) == Some(true) {
                return Some(range.catch_start as u64);
            }
        }

        None
    }
}

impl BufferedRead for CodeAttribute {
    fn read<T: Read + Seek>(buffer: &mut T) -> io::Result<Self> {
        let max_stack = buffer.read_u16::<BigEndian>()?;
        let max_locals = buffer.read_u16::<BigEndian>()?;

        let code_length = buffer.read_u32::<BigEndian>()?;
        let mut code = vec![0u8; code_length as usize];
        buffer.read_exact(&mut code)?;

        let reader = InstructionReader::new();

        Ok(CodeAttribute {
            max_stack,
            max_locals,
            instructions: reader.parse(&mut Cursor::new(code))?,
            exception_table: <Vec<ExceptionRange>>::read(buffer)?,
            attributes: <Vec<AttributeInfo>>::read(buffer)?,
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ExceptionRange {
    try_start: u16,
    try_end: u16,
    catch_start: u16,
    catch_type: u16,
}

impl BufferedRead for ExceptionRange {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        Ok(ExceptionRange {
            try_start: buffer.read_u16::<BigEndian>()?,
            try_end: buffer.read_u16::<BigEndian>()?,
            catch_start: buffer.read_u16::<BigEndian>()?,
            catch_type: buffer.read_u16::<BigEndian>()?,
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LineNumber {
    instruction: u16,
    line_num: u16,
}

impl BufferedRead for LineNumber {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        Ok(LineNumber {
            instruction: buffer.read_u16::<BigEndian>()?,
            line_num: buffer.read_u16::<BigEndian>()?,
        })
    }
}
