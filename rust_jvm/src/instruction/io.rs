use crate::instruction::Instruction::*;
use crate::instruction::{Instruction, PrimitiveType};
use byteorder::{BigEndian, NativeEndian, ReadBytesExt};
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
        match self {
            Instruction::nop => 0x00,
            Instruction::aconst_null => 0x01,
            Instruction::iconst_m1 => 0x02,
            Instruction::iconst_0 => 0x03,
            Instruction::iconst_1 => 0x04,
            Instruction::iconst_2 => 0x05,
            Instruction::iconst_3 => 0x06,
            Instruction::iconst_4 => 0x07,
            Instruction::iconst_5 => 0x08,
            Instruction::lconst_0 => 0x09,
            Instruction::lconst_1 => 0x0a,
            Instruction::fconst_0 => 0x0b,
            Instruction::fconst_1 => 0x0c,
            Instruction::fconst_2 => 0x0d,
            Instruction::dconst_0 => 0x0e,
            Instruction::dconst_1 => 0x0f,
            Instruction::bipush(_) => 0x10,
            Instruction::sipush(_) => 0x11,
            Instruction::ldc(_) => 0x12,
            Instruction::ldc_w(_) => 0x13,
            Instruction::ldc2_w(_) => 0x14,
            Instruction::iload(0) => 0x1a,
            Instruction::iload(1) => 0x1b,
            Instruction::iload(2) => 0x1c,
            Instruction::iload(3) => 0x1d,
            Instruction::iload(x) if *x <= u8::MAX as u16 => 0x15,
            Instruction::iload(_) => 0xc4,
            Instruction::lload(0) => 0x1e,
            Instruction::lload(1) => 0x1f,
            Instruction::lload(2) => 0x20,
            Instruction::lload(3) => 0x21,
            Instruction::lload(x) if *x <= u8::MAX as u16 => 0x16,
            Instruction::lload(_) => 0xc4,
            Instruction::fload(0) => 0x22,
            Instruction::fload(1) => 0x23,
            Instruction::fload(2) => 0x24,
            Instruction::fload(3) => 0x25,
            Instruction::fload(x) if *x <= u8::MAX as u16 => 0x17,
            Instruction::fload(_) => 0xc4,
            Instruction::dload(0) => 0x26,
            Instruction::dload(1) => 0x27,
            Instruction::dload(2) => 0x28,
            Instruction::dload(3) => 0x29,
            Instruction::dload(x) if *x <= u8::MAX as u16 => 0x18,
            Instruction::dload(_) => 0xc4,
            Instruction::aload(0) => 0x2a,
            Instruction::aload(1) => 0x2b,
            Instruction::aload(2) => 0x2c,
            Instruction::aload(3) => 0x2d,
            Instruction::aload(x) if *x <= u8::MAX as u16 => 0x19,
            Instruction::aload(_) => 0xc4,
            Instruction::iaload => 0x2e,
            Instruction::laload => 0x2f,
            Instruction::faload => 0x30,
            Instruction::daload => 0x31,
            Instruction::aaload => 0x32,
            Instruction::baload => 0x33,
            Instruction::caload => 0x34,
            Instruction::saload => 0x35,
            Instruction::istore(0) => 0x3b,
            Instruction::istore(1) => 0x3c,
            Instruction::istore(2) => 0x3d,
            Instruction::istore(3) => 0x3e,
            Instruction::istore(x) if *x <= u8::MAX as u16 => 0x36,
            Instruction::istore(_) => 0xc4,
            Instruction::lstore(0) => 0x3f,
            Instruction::lstore(1) => 0x40,
            Instruction::lstore(2) => 0x41,
            Instruction::lstore(3) => 0x42,
            Instruction::lstore(x) if *x <= u8::MAX as u16 => 0x37,
            Instruction::lstore(_) => 0xc4,
            Instruction::fstore(0) => 0x43,
            Instruction::fstore(1) => 0x44,
            Instruction::fstore(2) => 0x45,
            Instruction::fstore(3) => 0x46,
            Instruction::fstore(x) if *x <= u8::MAX as u16 => 0x38,
            Instruction::fstore(_) => 0xc4,
            Instruction::dstore(0) => 0x47,
            Instruction::dstore(1) => 0x48,
            Instruction::dstore(2) => 0x49,
            Instruction::dstore(3) => 0x4a,
            Instruction::dstore(x) if *x <= u8::MAX as u16 => 0x39,
            Instruction::dstore(_) => 0xc4,
            Instruction::astore(0) => 0x4b,
            Instruction::astore(1) => 0x4c,
            Instruction::astore(2) => 0x4d,
            Instruction::astore(3) => 0x4e,
            Instruction::astore(x) if *x <= u8::MAX as u16 => 0x3a,
            Instruction::astore(_) => 0xc4,
            Instruction::iastore => 0x4f,
            Instruction::lastore => 0x50,
            Instruction::fastore => 0x51,
            Instruction::dastore => 0x52,
            Instruction::aastore => 0x53,
            Instruction::bastore => 0x54,
            Instruction::castore => 0x55,
            Instruction::sastore => 0x56,
            Instruction::pop => 0x57,
            Instruction::pop2 => 0x58,
            Instruction::dup => 0x59,
            Instruction::dup_x1 => 0x5a,
            Instruction::dup_x2 => 0x5b,
            Instruction::dup2 => 0x5c,
            Instruction::dup2_x1 => 0x5d,
            Instruction::dup2_x2 => 0x5e,
            Instruction::swap => 0x5f,
            Instruction::iadd => 0x60,
            Instruction::ladd => 0x61,
            Instruction::fadd => 0x62,
            Instruction::dadd => 0x63,
            Instruction::isub => 0x64,
            Instruction::lsub => 0x65,
            Instruction::fsub => 0x66,
            Instruction::dsub => 0x67,
            Instruction::imul => 0x68,
            Instruction::lmul => 0x69,
            Instruction::fmul => 0x6a,
            Instruction::dmul => 0x6b,
            Instruction::idiv => 0x6c,
            Instruction::ldiv => 0x6d,
            Instruction::fdiv => 0x6e,
            Instruction::ddiv => 0x6f,
            Instruction::irem => 0x70,
            Instruction::lrem => 0x71,
            Instruction::frem => 0x72,
            Instruction::drem => 0x73,
            Instruction::ineg => 0x74,
            Instruction::lneg => 0x75,
            Instruction::fneg => 0x76,
            Instruction::dneg => 0x77,
            Instruction::ishl => 0x78,
            Instruction::lshl => 0x79,
            Instruction::ishr => 0x7a,
            Instruction::lshr => 0x7b,
            Instruction::iushr => 0x7c,
            Instruction::lushr => 0x7d,
            Instruction::iand => 0x7e,
            Instruction::land => 0x7f,
            Instruction::ior => 0x80,
            Instruction::lor => 0x81,
            Instruction::ixor => 0x82,
            Instruction::lxor => 0x83,
            Instruction::iinc { index, const_inc }
                if *index <= u8::MAX as u16
                    && *const_inc <= i8::MAX as i16
                    && *const_inc >= i8::MIN as i16 =>
            {
                0x84
            }
            Instruction::iinc { .. } => 0xc4,
            Instruction::i2l => 0x85,
            Instruction::i2f => 0x86,
            Instruction::i2d => 0x87,
            Instruction::l2i => 0x88,
            Instruction::l2f => 0x89,
            Instruction::l2d => 0x8a,
            Instruction::f2i => 0x8b,
            Instruction::f2l => 0x8c,
            Instruction::f2d => 0x8d,
            Instruction::d2i => 0x8e,
            Instruction::d2l => 0x8f,
            Instruction::d2f => 0x90,
            Instruction::i2b => 0x91,
            Instruction::i2c => 0x92,
            Instruction::i2s => 0x93,
            Instruction::lcmp => 0x94,
            Instruction::fcmpl => 0x95,
            Instruction::fcmpg => 0x96,
            Instruction::dcmpl => 0x97,
            Instruction::dcmpg => 0x98,
            Instruction::ifeq(_) => 0x99,
            Instruction::ifne(_) => 0x9a,
            Instruction::iflt(_) => 0x9b,
            Instruction::ifge(_) => 0x9c,
            Instruction::ifgt(_) => 0x9d,
            Instruction::ifle(_) => 0x9e,
            Instruction::if_icmpeq(_) => 0x9f,
            Instruction::if_icmpne(_) => 0xa0,
            Instruction::if_icmplt(_) => 0xa1,
            Instruction::if_icmpge(_) => 0xa2,
            Instruction::if_icmpgt(_) => 0xa3,
            Instruction::if_icmple(_) => 0xa4,
            Instruction::if_acmpeq(_) => 0xa5,
            Instruction::if_acmpne(_) => 0xa6,
            Instruction::goto(_) => 0xa7,
            Instruction::jsr(_) => 0xa8,
            Instruction::ret(x) if *x <= u8::MAX as u16 => 0xa9,
            Instruction::ret(_) => 0xc4,
            Instruction::tableswitch { .. } => 0xaa,
            Instruction::lookupswitch { .. } => 0xab,
            Instruction::ireturn => 0xac,
            Instruction::lreturn => 0xad,
            Instruction::freturn => 0xae,
            Instruction::dreturn => 0xaf,
            Instruction::areturn => 0xb0,
            Instruction::r#return => 0xb1,
            Instruction::getstatic(_) => 0xb2,
            Instruction::putstatic(_) => 0xb3,
            Instruction::getfield(_) => 0xb4,
            Instruction::putfield(_) => 0xb5,
            Instruction::invokevirtual(_) => 0xb6,
            Instruction::invokespecial(_) => 0xb7,
            Instruction::invokestatic(_) => 0xb8,
            Instruction::invokeinterface { .. } => 0xb9,
            Instruction::invokedynamic(_) => 0xba,
            Instruction::new(_) => 0xbb,
            Instruction::newarray(_) => 0xbc,
            Instruction::anewarray(_) => 0xbd,
            Instruction::arraylength => 0xbe,
            Instruction::athrow => 0xbf,
            Instruction::checkcast(_) => 0xc0,
            Instruction::instanceof(_) => 0xc1,
            Instruction::monitorenter => 0xc2,
            Instruction::monitorexit => 0xc3,
            // Instruction::wide => 0xc4,
            Instruction::multianewarray { .. } => 0xc5,
            Instruction::ifnull(_) => 0xc6,
            Instruction::ifnonnull(_) => 0xc7,
            Instruction::goto_w(_) => 0xc8,
            Instruction::jsr_w(_) => 0xc9,
        }
    }

    pub fn write_single<W: Write + Seek>(buf: &mut W) -> io::Result<Self> {
        todo!()
    }

    pub fn parse_single<R: Read + Seek>(buf: &mut R, opcode: u8) -> io::Result<Self> {
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
