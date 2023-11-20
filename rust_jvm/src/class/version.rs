use std::cmp::Ordering;
use std::io::{self, Cursor, Read, Write};

use crate::class::BufferedRead;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

pub fn check_magic_number(buffer: &mut Cursor<Vec<u8>>) -> io::Result<bool> {
    let magic = buffer.read_u32::<BigEndian>()?;
    // println!("Magic Number: {:x}", magic);
    Ok(magic == 0xCAFEBABE)
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ClassVersion(pub u16, pub u16);

impl BufferedRead for ClassVersion {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        Ok(ClassVersion(
            buffer.read_u16::<BigEndian>()?,
            buffer.read_u16::<BigEndian>()?,
        ))
    }

    fn write<T: Write>(&self, buffer: &mut T) -> io::Result<()> {
        let ClassVersion(major, minor) = *self;

        buffer.write_u16::<BigEndian>(major)?;
        buffer.write_u16::<BigEndian>(minor)
    }
}

impl ClassVersion {
    pub const fn new(major: u16, minor: u16) -> Self {
        ClassVersion(major, minor)
    }
}

impl PartialOrd for ClassVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ClassVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.cmp(&other.0) {
            Ordering::Equal => self.1.cmp(&other.1),
            x => x,
        }
    }
}
