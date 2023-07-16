use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::{Cursor, Read, Seek, Write};

pub trait DebugWithConst: Sized {
    fn fmt(&self, f: &mut Formatter<'_>, pool: &ConstantPool<'_>) -> std::fmt::Result;

    fn tabbed_fmt(
        &self,
        f: &mut Formatter<'_>,
        pool: &ConstantPool<'_>,
        tabs: usize,
    ) -> std::fmt::Result {
        let out = format!("{}", self.display(pool));
        let offset = "  ".repeat(tabs);
        write!(
            f,
            "{}{}",
            &offset,
            out.replace('\n', &("\n".to_string() + &offset))
        )
    }

    fn display<'a, 'b: 'a>(
        &'a self,
        pool: &'a ConstantPool<'b>,
    ) -> DebugWithConstDisplay<'a, 'b, Self> {
        DebugWithConstDisplay(self, pool)
    }
}

pub struct DebugWithConstDisplay<'a, 'b, T>(&'a T, &'a ConstantPool<'b>);

impl<'a, 'b, T: DebugWithConst> Display for DebugWithConstDisplay<'a, 'b, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f, self.1)
    }
}

// TODO: Replace with more readable/usable version used in class_format
macro_rules! readable_struct {
    (pub struct $name:ident { $($field:ident: $type:ty),* $(,)? }) => {
        #[derive(Debug, Copy, Clone)]
        pub struct $name {
            $(pub $field: $type),*
        }

        readable_struct!{@impl $name {$($field: $type),* }}
    };
    (pub no_copy struct $name:ident { $($field:ident: $type:ty),* $(,)? }) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $(pub $field: $type),*
        }

        readable_struct!{@impl $name {$($field: $type),* }}
    };
    (@impl $name:ident { $($field:ident: $type:ty),* }) => {
        #[allow(unused_variables)]
        impl BufferedRead for $name {
            fn read<T: Read + Seek>(buffer: &mut T) -> io::Result<Self> {
                Ok($name { $($field: <$type as BufferedRead>::read(buffer)?),* })
            }

            fn write<T: Write + Seek>(&self, buffer: &mut T) -> io::Result<()> {
                $(<$type as BufferedRead>::write(&self.$field, buffer)?;)*
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

#[cfg(feature = "llvm")]
pub mod llvm;

use crate::class::constant::ConstantPool;
pub use class_file::*;
pub use load::*;
use std::fmt::{Display, Formatter};

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
