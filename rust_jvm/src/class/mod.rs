use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::{Cursor, Read, Seek, Write};

macro_rules! readable_struct {
    (pub struct $name:ident { $($field:ident: $type:ty),* $(,)? }) => {
        #[derive(Debug, Copy, Clone)]
        pub struct $name {
            $(pub $field: $type),+
        }

        impl BufferedRead for $name {
            fn read<T: Read + Seek>(buffer: &mut T) -> io::Result<Self> {
                Ok($name { $($field: <$type as BufferedRead>::read(buffer)?),+ })
            }

            fn write<T: Write + Seek>(&self, buffer: &mut T) -> io::Result<()> {
                $(<$type as BufferedRead>::write(&self.$field, buffer)?;)+
                Ok(())
            }
        }
    };
}

pub mod attribute;
mod class_file;
pub mod constant;
mod jar;
mod load;
mod version;

pub use class_file::*;
pub use load::*;

pub trait BufferedRead: Sized {
    fn read_str(string: &str) -> io::Result<Self> {
        let mut buffer = Cursor::new(string.as_bytes().to_vec());
        Self::read(&mut buffer)
    }

    fn read<T: Read + Seek>(buffer: &mut T) -> io::Result<Self>;

    fn write<T: Write + Seek>(&self, _: &mut T) -> io::Result<()> {
        unimplemented!("Write has not yet been implemented for this struct!")
    }
}

impl<T: BufferedRead> BufferedRead for Vec<T> {
    fn read<B: Read + Seek>(buffer: &mut B) -> io::Result<Self> {
        let count = buffer.read_u16::<BigEndian>()?;
        let mut vec = Vec::with_capacity(count as usize);

        for _ in 0..count {
            vec.push(T::read(buffer)?);
        }

        Ok(vec)
    }

    fn write<B: Write + Seek>(&self, buffer: &mut B) -> io::Result<()> {
        buffer.write_u16::<BigEndian>(self.len() as u16)?;

        for value in self {
            value.write(buffer)?;
        }

        Ok(())
    }
}

impl BufferedRead for u8 {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        buffer.read_u8()
    }
    fn write<T: Write>(&self, buffer: &mut T) -> io::Result<()> {
        buffer.write_u8(*self)
    }
}

impl BufferedRead for u16 {
    fn read<T: Read + Seek>(buffer: &mut T) -> io::Result<Self> {
        buffer.read_u16::<BigEndian>()
    }

    fn write<T: Write>(&self, buffer: &mut T) -> io::Result<()> {
        buffer.write_u16::<BigEndian>(*self)
    }
}

impl BufferedRead for i64 {
    fn read<T: Read + Seek>(buffer: &mut T) -> io::Result<Self> {
        buffer.read_i64::<BigEndian>()
    }
    fn write<T: Write>(&self, buffer: &mut T) -> io::Result<()> {
        buffer.write_i64::<BigEndian>(*self)
    }
}

impl BufferedRead for f64 {
    fn read<T: Read + Seek>(buffer: &mut T) -> io::Result<Self> {
        buffer.read_f64::<BigEndian>()
    }
    fn write<T: Write>(&self, buffer: &mut T) -> io::Result<()> {
        buffer.write_f64::<BigEndian>(*self)
    }
}

impl BufferedRead for i32 {
    fn read<T: Read + Seek>(buffer: &mut T) -> io::Result<Self> {
        buffer.read_i32::<BigEndian>()
    }
    fn write<T: Write>(&self, buffer: &mut T) -> io::Result<()> {
        buffer.write_i32::<BigEndian>(*self)
    }
}

impl BufferedRead for f32 {
    fn read<T: Read + Seek>(buffer: &mut T) -> io::Result<Self> {
        buffer.read_f32::<BigEndian>()
    }
    fn write<T: Write>(&self, buffer: &mut T) -> io::Result<()> {
        buffer.write_f32::<BigEndian>(*self)
    }
}
