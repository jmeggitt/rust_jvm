use crate::instruction::{Instruction, PrimitiveType};
use byteorder::{BigEndian, NativeEndian, ReadBytesExt, WriteBytesExt};
use std::convert::TryFrom;
use std::io;
use std::io::ErrorKind::InvalidData;
use std::io::{Error, Read, Seek, Write};

impl Instruction {
    pub fn parse_method_body<R, I>(buf: &mut R, instruction_buf: &mut I) -> io::Result<()>
    where
        R: Read + Seek,
        I: Extend<(u64, Self)>,
    {
        let mut opcode = 0;
        while buf.read(std::slice::from_mut(&mut opcode))? == 1 {
            let index = buf.stream_position()? - 1;
            let instruction = Self::parse_single(buf, opcode)?;
            instruction_buf.extend(Some((index, instruction)));
        }

        Ok(())
    }

    pub fn opcode(&self) -> u8 {
        use Instruction as I;

        match self {
            I::nop => 0x00,
            I::aconst_null => 0x01,
            I::iconst_m1 => 0x02,
            I::iconst_0 => 0x03,
            I::iconst_1 => 0x04,
            I::iconst_2 => 0x05,
            I::iconst_3 => 0x06,
            I::iconst_4 => 0x07,
            I::iconst_5 => 0x08,
            I::lconst_0 => 0x09,
            I::lconst_1 => 0x0a,
            I::fconst_0 => 0x0b,
            I::fconst_1 => 0x0c,
            I::fconst_2 => 0x0d,
            I::dconst_0 => 0x0e,
            I::dconst_1 => 0x0f,
            I::bipush(_) => 0x10,
            I::sipush(_) => 0x11,
            I::ldc(_) => 0x12,
            I::ldc_w(_) => 0x13,
            I::ldc2_w(_) => 0x14,
            I::iload(0) => 0x1a,
            I::iload(1) => 0x1b,
            I::iload(2) => 0x1c,
            I::iload(3) => 0x1d,
            I::iload(x) if *x <= u8::MAX as u16 => 0x15,
            I::iload(_) => 0xc4,
            I::lload(0) => 0x1e,
            I::lload(1) => 0x1f,
            I::lload(2) => 0x20,
            I::lload(3) => 0x21,
            I::lload(x) if *x <= u8::MAX as u16 => 0x16,
            I::lload(_) => 0xc4,
            I::fload(0) => 0x22,
            I::fload(1) => 0x23,
            I::fload(2) => 0x24,
            I::fload(3) => 0x25,
            I::fload(x) if *x <= u8::MAX as u16 => 0x17,
            I::fload(_) => 0xc4,
            I::dload(0) => 0x26,
            I::dload(1) => 0x27,
            I::dload(2) => 0x28,
            I::dload(3) => 0x29,
            I::dload(x) if *x <= u8::MAX as u16 => 0x18,
            I::dload(_) => 0xc4,
            I::aload(0) => 0x2a,
            I::aload(1) => 0x2b,
            I::aload(2) => 0x2c,
            I::aload(3) => 0x2d,
            I::aload(x) if *x <= u8::MAX as u16 => 0x19,
            I::aload(_) => 0xc4,
            I::iaload => 0x2e,
            I::laload => 0x2f,
            I::faload => 0x30,
            I::daload => 0x31,
            I::aaload => 0x32,
            I::baload => 0x33,
            I::caload => 0x34,
            I::saload => 0x35,
            I::istore(0) => 0x3b,
            I::istore(1) => 0x3c,
            I::istore(2) => 0x3d,
            I::istore(3) => 0x3e,
            I::istore(x) if *x <= u8::MAX as u16 => 0x36,
            I::istore(_) => 0xc4,
            I::lstore(0) => 0x3f,
            I::lstore(1) => 0x40,
            I::lstore(2) => 0x41,
            I::lstore(3) => 0x42,
            I::lstore(x) if *x <= u8::MAX as u16 => 0x37,
            I::lstore(_) => 0xc4,
            I::fstore(0) => 0x43,
            I::fstore(1) => 0x44,
            I::fstore(2) => 0x45,
            I::fstore(3) => 0x46,
            I::fstore(x) if *x <= u8::MAX as u16 => 0x38,
            I::fstore(_) => 0xc4,
            I::dstore(0) => 0x47,
            I::dstore(1) => 0x48,
            I::dstore(2) => 0x49,
            I::dstore(3) => 0x4a,
            I::dstore(x) if *x <= u8::MAX as u16 => 0x39,
            I::dstore(_) => 0xc4,
            I::astore(0) => 0x4b,
            I::astore(1) => 0x4c,
            I::astore(2) => 0x4d,
            I::astore(3) => 0x4e,
            I::astore(x) if *x <= u8::MAX as u16 => 0x3a,
            I::astore(_) => 0xc4,
            I::iastore => 0x4f,
            I::lastore => 0x50,
            I::fastore => 0x51,
            I::dastore => 0x52,
            I::aastore => 0x53,
            I::bastore => 0x54,
            I::castore => 0x55,
            I::sastore => 0x56,
            I::pop => 0x57,
            I::pop2 => 0x58,
            I::dup => 0x59,
            I::dup_x1 => 0x5a,
            I::dup_x2 => 0x5b,
            I::dup2 => 0x5c,
            I::dup2_x1 => 0x5d,
            I::dup2_x2 => 0x5e,
            I::swap => 0x5f,
            I::iadd => 0x60,
            I::ladd => 0x61,
            I::fadd => 0x62,
            I::dadd => 0x63,
            I::isub => 0x64,
            I::lsub => 0x65,
            I::fsub => 0x66,
            I::dsub => 0x67,
            I::imul => 0x68,
            I::lmul => 0x69,
            I::fmul => 0x6a,
            I::dmul => 0x6b,
            I::idiv => 0x6c,
            I::ldiv => 0x6d,
            I::fdiv => 0x6e,
            I::ddiv => 0x6f,
            I::irem => 0x70,
            I::lrem => 0x71,
            I::frem => 0x72,
            I::drem => 0x73,
            I::ineg => 0x74,
            I::lneg => 0x75,
            I::fneg => 0x76,
            I::dneg => 0x77,
            I::ishl => 0x78,
            I::lshl => 0x79,
            I::ishr => 0x7a,
            I::lshr => 0x7b,
            I::iushr => 0x7c,
            I::lushr => 0x7d,
            I::iand => 0x7e,
            I::land => 0x7f,
            I::ior => 0x80,
            I::lor => 0x81,
            I::ixor => 0x82,
            I::lxor => 0x83,
            I::iinc { index, const_inc }
                if *index <= u8::MAX as u16
                    && *const_inc <= i8::MAX as i16
                    && *const_inc >= i8::MIN as i16 =>
            {
                0x84
            }
            I::iinc { .. } => 0xc4,
            I::i2l => 0x85,
            I::i2f => 0x86,
            I::i2d => 0x87,
            I::l2i => 0x88,
            I::l2f => 0x89,
            I::l2d => 0x8a,
            I::f2i => 0x8b,
            I::f2l => 0x8c,
            I::f2d => 0x8d,
            I::d2i => 0x8e,
            I::d2l => 0x8f,
            I::d2f => 0x90,
            I::i2b => 0x91,
            I::i2c => 0x92,
            I::i2s => 0x93,
            I::lcmp => 0x94,
            I::fcmpl => 0x95,
            I::fcmpg => 0x96,
            I::dcmpl => 0x97,
            I::dcmpg => 0x98,
            I::ifeq(_) => 0x99,
            I::ifne(_) => 0x9a,
            I::iflt(_) => 0x9b,
            I::ifge(_) => 0x9c,
            I::ifgt(_) => 0x9d,
            I::ifle(_) => 0x9e,
            I::if_icmpeq(_) => 0x9f,
            I::if_icmpne(_) => 0xa0,
            I::if_icmplt(_) => 0xa1,
            I::if_icmpge(_) => 0xa2,
            I::if_icmpgt(_) => 0xa3,
            I::if_icmple(_) => 0xa4,
            I::if_acmpeq(_) => 0xa5,
            I::if_acmpne(_) => 0xa6,
            I::goto(_) => 0xa7,
            I::jsr(_) => 0xa8,
            I::ret(x) if *x <= u8::MAX as u16 => 0xa9,
            I::ret(_) => 0xc4,
            I::tableswitch { .. } => 0xaa,
            I::lookupswitch { .. } => 0xab,
            I::ireturn => 0xac,
            I::lreturn => 0xad,
            I::freturn => 0xae,
            I::dreturn => 0xaf,
            I::areturn => 0xb0,
            I::r#return => 0xb1,
            I::getstatic(_) => 0xb2,
            I::putstatic(_) => 0xb3,
            I::getfield(_) => 0xb4,
            I::putfield(_) => 0xb5,
            I::invokevirtual(_) => 0xb6,
            I::invokespecial(_) => 0xb7,
            I::invokestatic(_) => 0xb8,
            I::invokeinterface { .. } => 0xb9,
            I::invokedynamic(_) => 0xba,
            I::new(_) => 0xbb,
            I::newarray(_) => 0xbc,
            I::anewarray(_) => 0xbd,
            I::arraylength => 0xbe,
            I::athrow => 0xbf,
            I::checkcast(_) => 0xc0,
            I::instanceof(_) => 0xc1,
            I::monitorenter => 0xc2,
            I::monitorexit => 0xc3,
            // I::wide => 0xc4,
            I::multianewarray { .. } => 0xc5,
            I::ifnull(_) => 0xc6,
            I::ifnonnull(_) => 0xc7,
            I::goto_w(_) => 0xc8,
            I::jsr_w(_) => 0xc9,
        }
    }

    pub fn write_single<W: Write + Seek>(&self, buf: &mut W) -> io::Result<()> {
        use Instruction as I;
        buf.write_u8(self.opcode())?;

        match self {
            // Instructions which get modified by the wide opcode
            I::iload(x)
            | I::lload(x)
            | I::fload(x)
            | I::dload(x)
            | I::aload(x)
            | I::istore(x)
            | I::lstore(x)
            | I::fstore(x)
            | I::dstore(x)
            | I::astore(x)
                if *x > 3 =>
            {
                if let Ok(v) = u8::try_from(*x) {
                    buf.write_u8(v)
                } else {
                    // The opcode we wrote before was the wide instruction opcode. We need to write
                    // the base opcode of this instruction to specify which variant we are using.
                    let base_opcode = match self {
                        I::iload(_) => 0x15,
                        I::lload(_) => 0x16,
                        I::fload(_) => 0x17,
                        I::dload(_) => 0x18,
                        I::aload(_) => 0x19,
                        I::istore(_) => 0x36,
                        I::lstore(_) => 0x37,
                        I::fstore(_) => 0x38,
                        I::dstore(_) => 0x39,
                        I::astore(_) => 0x3a,
                        _ => unreachable!("Should not reach this point due to parent match"),
                    };

                    buf.write_u8(base_opcode)?;
                    buf.write_u16::<BigEndian>(*x)
                }
            }
            I::ret(x) => {
                if let Ok(v) = u8::try_from(*x) {
                    buf.write_u8(v)
                } else {
                    buf.write_u8(0xa9)?;
                    buf.write_u16::<BigEndian>(*x)
                }
            }
            I::iinc { index, const_inc }
                if *index <= u8::MAX as u16
                    && *const_inc <= i8::MAX as i16
                    && *const_inc >= i8::MIN as i16 =>
            {
                buf.write_u8(*index as u8)?;
                buf.write_i8(*const_inc as i8)
            }
            I::iinc { index, const_inc } => {
                buf.write_u8(0x84)?;
                buf.write_u16::<BigEndian>(*index)?;
                buf.write_i16::<BigEndian>(*const_inc)
            }

            // Instructions which use an index into the constant pool
            I::anewarray(index)
            | I::checkcast(index)
            | I::getfield(index)
            | I::getstatic(index)
            | I::instanceof(index)
            | I::invokedynamic(index)
            | I::invokespecial(index)
            | I::invokestatic(index)
            | I::invokevirtual(index)
            | I::new(index)
            | I::putfield(index)
            | I::putstatic(index)
            | I::ldc_w(index)
            | I::ldc2_w(index) => buf.write_u16::<BigEndian>(*index),

            // Instructions which branch to a specific byte offset
            I::goto(offset)
            | I::if_acmpeq(offset)
            | I::if_acmpne(offset)
            | I::if_icmpeq(offset)
            | I::if_icmpne(offset)
            | I::if_icmplt(offset)
            | I::if_icmpge(offset)
            | I::if_icmpgt(offset)
            | I::if_icmple(offset)
            | I::ifeq(offset)
            | I::ifne(offset)
            | I::iflt(offset)
            | I::ifge(offset)
            | I::ifgt(offset)
            | I::ifle(offset)
            | I::ifnonnull(offset)
            | I::ifnull(offset)
            | I::jsr(offset) => buf.write_i16::<BigEndian>(*offset),

            // Misc instructions
            I::bipush(x) => buf.write_i8(*x),
            I::newarray(ty) => {
                let ty_byte = match ty {
                    PrimitiveType::Boolean => 4,
                    PrimitiveType::Char => 5,
                    PrimitiveType::Float => 6,
                    PrimitiveType::Double => 7,
                    PrimitiveType::Byte => 8,
                    PrimitiveType::Short => 9,
                    PrimitiveType::Int => 10,
                    PrimitiveType::Long => 11,
                };

                buf.write_u8(ty_byte)
            }
            I::goto_w(offset) | I::jsr_w(offset) => buf.write_i32::<BigEndian>(*offset),
            I::ldc(index) => buf.write_u8(*index),
            I::multianewarray { index, dimensions } => {
                buf.write_u16::<BigEndian>(*index)?;
                buf.write_u8(*dimensions)
            }
            I::sipush(x) => buf.write_i16::<BigEndian>(*x),
            I::invokeinterface { index, count } => {
                buf.write_u16::<BigEndian>(*index)?;
                buf.write_u8(*count)?;
                buf.write_u8(0)
            }
            I::lookupswitch {
                default_offset,
                match_offset,
            } => {
                while buf.stream_position()? % 4 != 0 {
                    buf.write_u8(0)?;
                }

                buf.write_i32::<BigEndian>(*default_offset)?;
                buf.write_u32::<BigEndian>(match_offset.len() as u32)?;
                for (value, offset) in match_offset {
                    buf.write_i32::<BigEndian>(*value)?;
                    buf.write_i32::<BigEndian>(*offset)?;
                }

                Ok(())
            }
            I::tableswitch {
                default_offset,
                low,
                jump_offsets,
            } => {
                while buf.stream_position()? % 4 != 0 {
                    buf.write_u8(0)?;
                }

                buf.write_i32::<BigEndian>(*default_offset)?;
                buf.write_i32::<BigEndian>(*low)?;
                buf.write_i32::<BigEndian>(*low + (jump_offsets.len() as i32) - 1)?;
                for offset in jump_offsets {
                    buf.write_i32::<BigEndian>(*offset)?;
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn parse_single<R: Read + Seek>(buf: &mut R, opcode: u8) -> io::Result<Self> {
        use crate::instruction::Instruction::*;

        let instruction = match opcode {
            0x32 => aaload,
            0x53 => aastore,
            0x01 => aconst_null,
            0x19 => aload(buf.read_u8()? as u16),
            0x2a => aload(0),
            0x2b => aload(1),
            0x2c => aload(2),
            0x2d => aload(3),
            0xbd => anewarray(buf.read_u16::<BigEndian>()?),
            0xb0 => areturn,
            0xbe => arraylength,
            0x3a => astore(buf.read_u8()? as u16),
            0x4b => astore(0),
            0x4c => astore(1),
            0x4d => astore(2),
            0x4e => astore(3),
            0xbf => athrow,
            0x33 => baload,
            0x54 => bastore,
            0x10 => bipush(buf.read_i8()?),
            0x34 => caload,
            0x55 => castore,
            0xc0 => checkcast(buf.read_u16::<BigEndian>()?),
            0x90 => d2f,
            0x8e => d2i,
            0x8f => d2l,
            0x63 => dadd,
            0x31 => daload,
            0x52 => dastore,
            0x98 => dcmpg,
            0x97 => dcmpl,
            0x0e => dconst_0,
            0x0f => dconst_1,
            0x6f => ddiv,
            0x18 => dload(buf.read_u8()? as u16),
            0x26 => dload(0),
            0x27 => dload(1),
            0x28 => dload(2),
            0x29 => dload(3),
            0x6b => dmul,
            0x77 => dneg,
            0x73 => drem,
            0xaf => dreturn,
            0x39 => dstore(buf.read_u8()? as u16),
            0x47 => dstore(0),
            0x48 => dstore(1),
            0x49 => dstore(2),
            0x4a => dstore(3),
            0x67 => dsub,
            0x59 => dup,
            0x5a => dup_x1,
            0x5b => dup_x2,
            0x5c => dup2,
            0x5d => dup2_x1,
            0x5e => dup2_x2,
            0x8d => f2d,
            0x8b => f2i,
            0x8c => f2l,
            0x62 => fadd,
            0x30 => faload,
            0x51 => fastore,
            0x96 => fcmpg,
            0x95 => fcmpl,
            0x0b => fconst_0,
            0x0c => fconst_1,
            0x0d => fconst_2,
            0x6e => fdiv,
            0x17 => fload(buf.read_u8()? as u16),
            0x22 => fload(0),
            0x23 => fload(1),
            0x24 => fload(2),
            0x25 => fload(3),
            0x6a => fmul,
            0x76 => fneg,
            0x72 => frem,
            0xae => freturn,
            0x38 => fstore(buf.read_u8()? as u16),
            0x43 => fstore(0),
            0x44 => fstore(1),
            0x45 => fstore(2),
            0x46 => fstore(3),
            0x66 => fsub,
            0xb4 => getfield(buf.read_u16::<BigEndian>()?),
            0xb2 => getstatic(buf.read_u16::<BigEndian>()?),
            0xa7 => goto(buf.read_i16::<BigEndian>()?),
            0xc8 => goto_w(buf.read_i32::<BigEndian>()?),
            0x91 => i2b,
            0x92 => i2c,
            0x87 => i2d,
            0x86 => i2f,
            0x85 => i2l,
            0x93 => i2s,
            0x60 => iadd,
            0x2e => iaload,
            0x7e => iand,
            0x4f => iastore,
            0x02 => iconst_m1,
            0x03 => iconst_0,
            0x04 => iconst_1,
            0x05 => iconst_2,
            0x06 => iconst_3,
            0x07 => iconst_4,
            0x08 => iconst_5,
            0x6c => idiv,
            0xa5 => if_acmpeq(buf.read_i16::<BigEndian>()?),
            0xa6 => if_acmpne(buf.read_i16::<BigEndian>()?),
            0x9f => if_icmpeq(buf.read_i16::<BigEndian>()?),
            0xa0 => if_icmpne(buf.read_i16::<BigEndian>()?),
            0xa1 => if_icmplt(buf.read_i16::<BigEndian>()?),
            0xa2 => if_icmpge(buf.read_i16::<BigEndian>()?),
            0xa3 => if_icmpgt(buf.read_i16::<BigEndian>()?),
            0xa4 => if_icmple(buf.read_i16::<BigEndian>()?),
            0x99 => ifeq(buf.read_i16::<BigEndian>()?),
            0x9a => ifne(buf.read_i16::<BigEndian>()?),
            0x9b => iflt(buf.read_i16::<BigEndian>()?),
            0x9c => ifge(buf.read_i16::<BigEndian>()?),
            0x9d => ifgt(buf.read_i16::<BigEndian>()?),
            0x9e => ifle(buf.read_i16::<BigEndian>()?),
            0xc7 => ifnonnull(buf.read_i16::<BigEndian>()?),
            0xc6 => ifnull(buf.read_i16::<BigEndian>()?),
            0x84 => iinc {
                index: buf.read_u8()? as u16,
                const_inc: buf.read_i8()? as i16,
            },
            0x15 => iload(buf.read_u8()? as u16),
            0x1a => iload(0),
            0x1b => iload(1),
            0x1c => iload(2),
            0x1d => iload(3),
            0x68 => imul,
            0x74 => ineg,
            0xc1 => instanceof(buf.read_u16::<BigEndian>()?),
            0xba => {
                let index = buf.read_u16::<BigEndian>()?;
                if buf.read_u16::<NativeEndian>()? != 0 {
                    return Err(Error::new(
                        InvalidData,
                        "Reserved bytes in invokedynamic must be 0",
                    ));
                }

                invokedynamic(index)
            }
            0xb9 => {
                let index = buf.read_u16::<BigEndian>()?;
                let count = buf.read_u8()?;
                if buf.read_u8()? != 0 {
                    return Err(Error::new(
                        InvalidData,
                        "Reserved byte in invokeinterface must be 0",
                    ));
                }

                invokeinterface { index, count }
            }
            0xb7 => invokespecial(buf.read_u16::<BigEndian>()?),
            0xb8 => invokestatic(buf.read_u16::<BigEndian>()?),
            0xb6 => invokevirtual(buf.read_u16::<BigEndian>()?),
            0x80 => ior,
            0x70 => irem,
            0xac => ireturn,
            0x78 => ishl,
            0x7a => ishr,
            0x36 => istore(buf.read_u8()? as u16),
            0x3b => istore(0),
            0x3c => istore(1),
            0x3d => istore(2),
            0x3e => istore(3),
            0x64 => isub,
            0x7c => iushr,
            0x82 => ixor,
            0xa8 => jsr(buf.read_i16::<BigEndian>()?),
            0xc9 => jsr_w(buf.read_i32::<BigEndian>()?),
            0x8a => l2d,
            0x89 => l2f,
            0x88 => l2i,
            0x61 => ladd,
            0x2f => laload,
            0x7f => land,
            0x50 => lastore,
            0x94 => lcmp,
            0x09 => lconst_0,
            0x0a => lconst_1,
            0x12 => ldc(buf.read_u8()?),
            0x13 => ldc_w(buf.read_u16::<BigEndian>()?),
            0x14 => ldc2_w(buf.read_u16::<BigEndian>()?),
            0x6d => ldiv,
            0x16 => lload(buf.read_u8()? as u16),
            0x1e => lload(0),
            0x1f => lload(1),
            0x20 => lload(2),
            0x21 => lload(3),
            0x69 => lmul,
            0x75 => lneg,
            0xab => {
                // Consume padding until we get to required alignment
                while buf.stream_position()? % 4 != 0 {
                    buf.read_u8()?;
                }

                let default_offset = buf.read_i32::<BigEndian>()?;
                let num_pairs = buf.read_i32::<BigEndian>()?;
                if num_pairs < 0 {
                    return Err(Error::new(
                        InvalidData,
                        "numpairs in lookupswitch must be >= 0",
                    ));
                }

                let mut match_offset = Vec::with_capacity(num_pairs as usize);
                for _ in 0..num_pairs {
                    match_offset.push((buf.read_i32::<BigEndian>()?, buf.read_i32::<BigEndian>()?));
                }

                lookupswitch {
                    default_offset,
                    match_offset,
                }
            }
            0x81 => lor,
            0x71 => lrem,
            0xad => lreturn,
            0x79 => lshl,
            0x7b => lshr,
            0x37 => lstore(buf.read_u8()? as u16),
            0x3f => lstore(0),
            0x40 => lstore(1),
            0x41 => lstore(2),
            0x42 => lstore(3),
            0x65 => lsub,
            0x7d => lushr,
            0x83 => lxor,
            0xc2 => monitorenter,
            0xc3 => monitorexit,
            0xc5 => multianewarray {
                index: buf.read_u16::<BigEndian>()?,
                dimensions: buf.read_u8()?,
            },
            0xbb => new(buf.read_u16::<BigEndian>()?),
            0xbc => match buf.read_u8()? {
                4 => newarray(PrimitiveType::Boolean),
                5 => newarray(PrimitiveType::Char),
                6 => newarray(PrimitiveType::Float),
                7 => newarray(PrimitiveType::Double),
                8 => newarray(PrimitiveType::Byte),
                9 => newarray(PrimitiveType::Short),
                10 => newarray(PrimitiveType::Int),
                11 => newarray(PrimitiveType::Long),
                _ => return Err(Error::new(InvalidData, "unknown array type")),
            },
            0x00 => nop,
            0x57 => pop,
            0x58 => pop2,
            0xb5 => putfield(buf.read_u16::<BigEndian>()?),
            0xb3 => putstatic(buf.read_u16::<BigEndian>()?),
            0xa9 => ret(buf.read_u8()? as u16),
            0xb1 => r#return,
            0x35 => saload,
            0x56 => sastore,
            0x11 => sipush(buf.read_i16::<BigEndian>()?),
            0x5f => swap,
            0xaa => {
                // Consume padding until we get to required alignment
                while buf.stream_position()? % 4 != 0 {
                    buf.read_u8()?;
                }

                let default_offset = buf.read_i32::<BigEndian>()?;
                let low = buf.read_i32::<BigEndian>()?;
                let high = buf.read_i32::<BigEndian>()?;
                let num_offsets = high - low + 1;
                if num_offsets < 0 {
                    return Err(Error::new(
                        InvalidData,
                        "number of offsets in tableswitch must be >= 0",
                    ));
                }

                let mut jump_offsets = Vec::with_capacity((high - low + 1) as usize);

                for _ in 0..(high - low + 1) {
                    jump_offsets.push(buf.read_i32::<BigEndian>()?);
                }

                tableswitch {
                    default_offset,
                    low,
                    jump_offsets,
                }
            }
            0xc4 => match buf.read_u8()? {
                0x15 => iload(buf.read_u16::<BigEndian>()?),
                0x17 => fload(buf.read_u16::<BigEndian>()?),
                0x19 => aload(buf.read_u16::<BigEndian>()?),
                0x16 => lload(buf.read_u16::<BigEndian>()?),
                0x18 => dload(buf.read_u16::<BigEndian>()?),
                0x36 => istore(buf.read_u16::<BigEndian>()?),
                0x38 => fstore(buf.read_u16::<BigEndian>()?),
                0x3a => astore(buf.read_u16::<BigEndian>()?),
                0x37 => lstore(buf.read_u16::<BigEndian>()?),
                0x39 => dstore(buf.read_u16::<BigEndian>()?),
                0xa9 => ret(buf.read_u16::<BigEndian>()?),
                0x84 => iinc {
                    index: buf.read_u16::<BigEndian>()?,
                    const_inc: buf.read_i16::<BigEndian>()?,
                },
                x => {
                    return Err(Error::new(
                        InvalidData,
                        format!("unknown opcode for wide instruction 0x{:02x}", x),
                    ));
                }
            },
            _ => return Err(Error::new(InvalidData, "unknown array type")),
        };

        Ok(instruction)
    }
}
