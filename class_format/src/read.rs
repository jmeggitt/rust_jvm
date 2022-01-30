use std::fmt::{Display, Formatter};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{self, Cursor, Read};
use std::ops::Deref;

/// A simple trait for streamlining the process of reading types from a generic source. Since this
/// trait is aimed solely at the Java Class file format it disregards other forms of encoding in
/// favor of a simplified interface. As such, all primitives as parsed in big endian unless
/// explicitly implemented otherwise for a given type.
pub trait Readable: Sized {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self>;

    // TODO: Is this helpful or is it just clutter?
    fn from_slice<T: AsRef<[u8]>>(slice: T) -> io::Result<Self> {
        let mut buffer = Cursor::new(slice.as_ref());
        Self::read(&mut buffer)
    }
}

#[macro_export]
macro_rules! simple_grammar {
    ($($(#[$($macros:tt)+])* $pub:vis struct $name:ident { $($(#[$($field_macros:tt)+])* $field_vis:vis $field:ident: $type:ty),* $(,)? })+) => {
        $(simple_grammar!{
            @impl $(#[$($macros)+])*
            $pub struct $name {
                $($(#[$($field_macros)+])*
                $field_vis $field: $type),*
            }
        })+
    };
    (@impl $(#[$($macros:tt)+])* $pub:vis struct $name:ident { $($(#[$($field_macros:tt)+])* $field_vis:vis $field:ident: $type:ty),* $(,)? }) => {
        $(#[$($macros)+])*
        $pub struct $name {
                $($(#[$($field_macros)+])*
                $field_vis $field: $type),*
        }

        impl Readable for $name {
            fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
                Ok($name { $($field: <$type as Readable>::read(buffer)?),+ })
            }
        }
    };
}

/// Most (with some notable exceptions) repetitions of structures in the class file follow the
/// format of having a u16 holding number of instances followed by the entries. This implementation
/// handles that general case. However it will not work for all structures (such as the constant
/// pool with has its own set of rules due to maintaining legacy support).
impl<T: Readable> Readable for Vec<T> {
    fn read<B: Read>(buffer: &mut B) -> io::Result<Self> {
        let count = buffer.read_u16::<BigEndian>()?;
        let mut vec = Vec::with_capacity(count as usize);

        for _ in 0..count {
            vec.push(T::read(buffer)?);
        }

        Ok(vec)
    }
}

/// Many structures in the class file specification will include a binary section for user defined
/// attributes or fields of varying length. This struct serves to support those areas. Another
/// option is `Vec<u8>`, however it reads the number of bytes as a u16 where as most binary
/// sections use a u32.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct BinarySection {
    section: Vec<u8>,
}

impl BinarySection {
    pub fn read_as<T: Readable>(&self) -> io::Result<T> {
        // TODO: Should not reading until the end of the buffer trigger a panic or error?
        let mut buffer = Cursor::new(&self.section);
        T::read(&mut buffer)
    }
}

impl Readable for BinarySection {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        let length = u32::read(buffer)? as usize;
        let mut section = vec![0; length];
        buffer.read_exact(&mut section)?;

        Ok(BinarySection { section })
    }
}

impl Deref for BinarySection {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.section
    }
}

// TODO: Also provide character view similar to hexdump
impl Display for BinarySection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut idx = 0;

        for byte in &self.section {
            if idx % 8 == 0 {
                write!(f, "{:06}", idx)?;
            }
            write!(f, " {:02X}", byte)?;

            idx += 1;
            if idx % 8 == 0 && idx != 0 && idx + 1 < self.section.len() {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

macro_rules! impl_primitive {
    ($type:ty: $($read:tt)+) => {
        impl Readable for $type {
            fn read<T: Read>(buffer: &mut T) -> io::Result<Self> { $($read)+(buffer) }
        }
    };
}

// Wrap primitives with byteorder read methods for big endian encoding.
impl_primitive!(u8: ReadBytesExt::read_u8);
impl_primitive!(u16: ReadBytesExt::read_u16::<BigEndian>);
impl_primitive!(u32: ReadBytesExt::read_u32::<BigEndian>);
impl_primitive!(u64: ReadBytesExt::read_u64::<BigEndian>);
impl_primitive!(i8: ReadBytesExt::read_i8);
impl_primitive!(i16: ReadBytesExt::read_i16::<BigEndian>);
impl_primitive!(i32: ReadBytesExt::read_i32::<BigEndian>);
impl_primitive!(i64: ReadBytesExt::read_i64::<BigEndian>);
impl_primitive!(f32: ReadBytesExt::read_f32::<BigEndian>);
impl_primitive!(f64: ReadBytesExt::read_f64::<BigEndian>);
