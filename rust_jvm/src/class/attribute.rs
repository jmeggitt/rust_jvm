use std::io;
use std::io::{Cursor, Read, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt};

use crate::class::class_file::AttributeInfo;
use crate::class::constant::ConstantPool;
use crate::class::{AccessFlags, BufferedRead, DebugWithConst};
use crate::instruction::Instruction;
use crate::instruction::InstructionReader;
use crate::jvm::JavaEnv;
use std::fmt::{Debug, Formatter};

#[derive(Debug)]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub instructions: Vec<(u64, Box<dyn Instruction>)>,
    pub exception_table: Vec<ExceptionRange>,
    pub attributes: Vec<AttributeInfo>,
}

impl DebugWithConst for CodeAttribute {
    fn fmt(&self, f: &mut Formatter<'_>, pool: &ConstantPool<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "CodeAttribute [{} stack; {} locals]",
            self.max_stack, self.max_locals
        )?;
        write!(f, "  Instructions:")?;
        for (idx, instruction) in &self.instructions {
            write!(f, "\n    {: <5}{:?}", format!("{}:", idx), instruction)?;
        }

        if !self.exception_table.is_empty() {
            write!(f, "\n  Exception Table:")?;
            for except in &self.exception_table {
                writeln!(f)?;
                except.tabbed_fmt(f, pool, 2)?;
                // writeln!(f, "  {:?}", except)?;
            }
        }

        if !self.attributes.is_empty() {
            write!(f, "\n  Attributes:")?;
            for attr in &self.attributes {
                writeln!(f)?;
                attr.tabbed_fmt(f, pool, 2)?;
                // writeln!(f, "  {}", format!("{:?}", attr).replace('\n', "\n  "))?;
            }
        }

        Ok(())
    }
}

impl CodeAttribute {
    pub fn attempt_catch(
        &self,
        pos: u64,
        class: &str,
        pool: &ConstantPool,
        jvm: &mut JavaEnv,
    ) -> Option<u64> {
        // I assume that the first one that fits is the one to use?
        for range in self.exception_table.iter().copied() {
            if pos < range.try_start as u64 || pos > range.try_end as u64 {
                continue;
            }

            let catch_target = pool.class_name(range.catch_type);
            // let index = pool[range.catch_type as usize - 1].expect_class().unwrap();
            // let catch_target = pool[index as usize - 1].expect_utf8().unwrap();

            if jvm.instanceof(class, catch_target) == Some(true) {
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

impl DebugWithConst for ExceptionRange {
    fn fmt(&self, f: &mut Formatter<'_>, pool: &ConstantPool<'_>) -> std::fmt::Result {
        writeln!(f, "ExceptionRange ({})", pool.class_name(self.catch_type))?;
        writeln!(f, "  Try: [{}, {}]", self.try_start, self.try_end)?;
        write!(f, "  Catch: {}", self.catch_start)
    }
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

readable_struct! {
    pub no_copy struct LineNumberTable {
        table: Vec<LineNumber>,
    }
}

impl DebugWithConst for LineNumberTable {
    fn fmt(&self, f: &mut Formatter<'_>, _pool: &ConstantPool<'_>) -> std::fmt::Result {
        write!(f, "LineNumberTable:")?;
        for line in &self.table {
            write!(
                f,
                "\n  {: <5}{}",
                format!("{}:", line.instruction),
                line.line_num
            )?;
        }

        Ok(())
    }
}

readable_struct! {
    pub struct SourceFile {
        index: u16,
    }
}

impl DebugWithConst for SourceFile {
    fn fmt(&self, f: &mut Formatter<'_>, pool: &ConstantPool<'_>) -> std::fmt::Result {
        write!(f, "SourceFile [{}]", pool.text(self.index))
    }
}

readable_struct! {
    pub struct EnclosingMethod {
        class_index: u16,
        method_index: u16,
    }
}

readable_struct! {
    pub struct NestHost {
        host_class_index: u16,
    }
}

readable_struct! {
    pub no_copy struct BootstrapMethod {
        bootstrap_method_ref: u16,
        bootstrap_arguments: Vec<u16>,
    }
}

readable_struct! {
    pub no_copy struct BootstrapMethods {
        methods: Vec<BootstrapMethod>,
    }
}

impl DebugWithConst for BootstrapMethods {
    fn fmt(&self, f: &mut Formatter<'_>, pool: &ConstantPool<'_>) -> std::fmt::Result {
        write!(f, "BootstrapMethods")?;
        for method in &self.methods {
            writeln!(f)?;
            pool[method.bootstrap_method_ref].tabbed_fmt(f, pool, 1)?;
            for arg in &method.bootstrap_arguments {
                writeln!(f)?;
                pool[*arg].tabbed_fmt(f, pool, 2)?;
            }
        }

        Ok(())
    }
}

readable_struct! {
    pub struct InnerClass {
        inner_class_info: u16,
        outer_class_info: u16,
        inner_name_index: u16,
        inner_class_access_flags: AccessFlags,
    }
}

readable_struct! {
    pub no_copy struct InnerClasses {
        classes: Vec<InnerClass>,
    }
}

impl DebugWithConst for InnerClasses {
    fn fmt(&self, f: &mut Formatter<'_>, pool: &ConstantPool<'_>) -> std::fmt::Result {
        write!(f, "InnerClasses")?;
        for inner in &self.classes {
            writeln!(f)?;
            if inner.inner_name_index == 0 {
                writeln!(
                    f,
                    "  <anonymous> ({}):",
                    pool.class_name(inner.inner_class_info)
                )?;
            } else {
                writeln!(
                    f,
                    "  {} ({}):",
                    pool.text(inner.inner_name_index),
                    pool.class_name(inner.inner_class_info)
                )?;
            }

            writeln!(f, "    Parent: {}", pool.class_name(inner.outer_class_info))?;
            write!(f, "    Access: {:?}", inner.inner_class_access_flags)?;
        }

        Ok(())
    }
}

readable_struct! {
    pub struct LocalVariableEntry {
        start_pc: u16,
        length: u16,
        name_index: u16,
        descriptor_index: u16,
        index: u16,
    }
}

readable_struct! {
    pub no_copy struct LocalVariableTable {
        variables: Vec<LocalVariableEntry>,
    }
}

impl DebugWithConst for LocalVariableTable {
    fn fmt(&self, f: &mut Formatter<'_>, pool: &ConstantPool<'_>) -> std::fmt::Result {
        write!(f, "LocalVariableTable")?;

        for variable in &self.variables {
            let pos = format!("{}:", variable.index);
            writeln!(f, "\n  {: <5}{}", pos, pool.text(variable.name_index))?;
            writeln!(
                f,
                "    Instructions: [{}, {}]",
                variable.start_pc,
                variable.start_pc + variable.index
            )?;
            write!(
                f,
                "    Descriptor: {}",
                pool.text(variable.descriptor_index)
            )?;
        }

        Ok(())
    }
}

readable_struct! {
    pub no_copy struct Exceptions {
        types: Vec<u16>,
    }
}

impl DebugWithConst for Exceptions {
    fn fmt(&self, f: &mut Formatter<'_>, pool: &ConstantPool<'_>) -> std::fmt::Result {
        write!(f, "Exceptions")?;

        for except in &self.types {
            write!(f, "\n  {}", pool.class_name(*except))?;
        }

        Ok(())
    }
}

readable_struct! {
    pub struct RuntimeVisibleAnnotations {}
}

impl DebugWithConst for RuntimeVisibleAnnotations {
    fn fmt(&self, f: &mut Formatter<'_>, _pool: &ConstantPool<'_>) -> std::fmt::Result {
        write!(f, "RuntimeVisibleAnnotations: TODO")
    }
}

readable_struct! {
    pub struct Signature {
        signature_index: u16,
    }
}

impl DebugWithConst for Signature {
    fn fmt(&self, f: &mut Formatter<'_>, pool: &ConstantPool<'_>) -> std::fmt::Result {
        write!(f, "Signature ({:?})", pool.text(self.signature_index))
    }
}

readable_struct! {
    pub struct StackMapTable {}
}

impl DebugWithConst for StackMapTable {
    fn fmt(&self, f: &mut Formatter<'_>, _pool: &ConstantPool<'_>) -> std::fmt::Result {
        write!(f, "StackMapTable: TODO")
    }
}
