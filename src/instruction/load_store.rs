//! This file contains all the loading and storing instructions. In other words, all of the
//! instructions supported by the wide instruction.

use crate::instruction::{Instruction, StaticInstruct};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor, Write};
use std::ops::RangeInclusive;

#[derive(Debug, Copy, Clone)]
pub struct iload(u8);

impl Instruction for iload {
    fn write(&self, buffer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        match self.0 {
            0..=3 => buffer.write_u8(0x1a + self.0)?,
            x => {
                buffer.write(&[0x15, x])?;
            }
        };
        Ok(())
    }
}

impl StaticInstruct for iload {
    const FORM: u8 = 0x15;
    const STRIDE: Option<RangeInclusive<u8>> = Some(0x1a..=0x1d);

    fn read(form: u8, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Box<dyn Instruction>> {
        Ok(Box::new(match form {
            0x15 => iload(buffer.read_u8()?),
            x => iload(x - 0x1a),
        }))
    }
}

// TODO: fload

#[derive(Debug, Copy, Clone)]
pub struct aload(u8);

impl Instruction for aload {
    fn write(&self, buffer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        match self.0 {
            0..=3 => buffer.write_u8(0x2a + self.0)?,
            x => {
                buffer.write(&[0x19, x])?;
            }
        }
        Ok(())
    }
}

impl StaticInstruct for aload {
    const FORM: u8 = 0x19;
    const STRIDE: Option<RangeInclusive<u8>> = Some(0x2a..=0x2d);

    fn read(form: u8, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Box<dyn Instruction>> {
        Ok(Box::new(match form {
            0x19 => aload(buffer.read_u8()?),
            x => aload(x - 0x2a),
        }))
    }
}

// TODO: lload

// TODO: dload

// TODO: istore

// TODO: fstore

#[derive(Debug, Copy, Clone)]
pub struct astore(u8);

impl Instruction for astore {
    fn write(&self, buffer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        match self.0 {
            0..=3 => buffer.write_u8(0x4b + self.0)?,
            x => {
                buffer.write(&[0x3a, x])?;
            }
        }
        Ok(())
    }
}

impl StaticInstruct for astore {
    const FORM: u8 = 0x3a;
    const STRIDE: Option<RangeInclusive<u8>> = Some(0x4b..=0x4e);

    fn read(form: u8, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Box<dyn Instruction>> {
        Ok(Box::new(match form {
            0x3a => astore(buffer.read_u8()?),
            x => astore(x - 0x4b),
        }))
    }
}

// TODO: lstore

// TODO: dstore

#[derive(Debug, Copy, Clone)]
pub struct ret(u8);

impl Instruction for ret {
    fn write(&self, buffer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        buffer.write_u8(Self::FORM)?;
        buffer.write_u8(self.0)
    }
}

impl StaticInstruct for ret {
    const FORM: u8 = 0xa9;

    fn read(_: u8, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Box<dyn Instruction>> {
        Ok(Box::new(ret(buffer.read_u8()?)))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct r#return;

impl Instruction for r#return {
    fn write(&self, buffer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        buffer.write_u8(Self::FORM)
    }
}

impl StaticInstruct for r#return {
    const FORM: u8 = 0xb1;

    fn read(_: u8, _: &mut Cursor<Vec<u8>>) -> io::Result<Box<dyn Instruction>> {
        Ok(Box::new(r#return))
    }
}

// TODO: wide
