use std::cmp::Ordering;
use std::io;
use std::io::{Cursor, Error, ErrorKind, Read, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bitflags::bitflags;

// use crate::class::attribute::{
//     BootstrapMethod, BootstrapMethods, CodeAttribute, EnclosingMethod, Exceptions, InnerClasses,
//     LineNumberTable, LocalVariableTable, NestHost, RuntimeVisibleAnnotations, Signature,
//     SourceFile, StackMapTable,
// };
// use crate::class::constant::{Constant, ConstantClass, ConstantPool};
// use crate::class::version::{check_magic_number, ClassVersion};
// use crate::class::{BufferedRead, DebugWithConst};
use std::fmt::Formatter;
use crate::attributes::AttributeInfo;
use crate::constant::{Constant, ConstantPool, RawConstantPool};
use crate::read::Readable;
use crate::simple_grammar;

bitflags! {
    pub struct ClassAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const FINAL = 0x0010;
        const SUPER = 0x0020;
        const INTERFACE = 0x0200;
        const ABSTRACT = 0x0400;
        const SYNTHETIC = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM = 0x4000;
        const MODULE = 0x8000;
    }
}

impl Readable for ClassAccessFlags {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        match ClassAccessFlags::from_bits(u16::read(buffer)?) {
            Some(v) => Ok(v),
            None => Err(Error::new(
                ErrorKind::InvalidInput,
                "Malformed ClassAccessFlags",
            )),
        }
    }
}
bitflags! {
    pub struct MethodAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const SYNCHRONIZED = 0x0020;
        const BRIDGE = 0x0040;
        const VARARGS = 0x0080;
        const NATIVE = 0x0100;
        const ABSTRACT = 0x0400;
        const STRICT = 0x0800;
        const SYNTHETIC = 0x1000;
    }
}


impl Readable for MethodAccessFlags {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        match MethodAccessFlags::from_bits(u16::read(buffer)?) {
            Some(v) => Ok(v),
            None => Err(Error::new(
                ErrorKind::InvalidInput,
                "Malformed ClassAccessFlags",
            )),
        }
    }
}

bitflags! {
    pub struct FieldAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const VOLATILE = 0x0040;
        const TRANSIENT = 0x0080;
        const SYNTHETIC = 0x1000;
        const ENUM = 0x4000;
    }
}

impl Readable for FieldAccessFlags {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        match FieldAccessFlags::from_bits(u16::read(buffer)?) {
            Some(v) => Ok(v),
            None => Err(Error::new(
                ErrorKind::InvalidInput,
                "Malformed ClassAccessFlags",
            )),
        }
    }
}

/// A placeholder which consumes 0xCAFEBABE on read to be used with the simple_grammar macro.
#[derive(Debug, Copy, Clone)]
struct ClassMagicBytes;

impl Readable for ClassMagicBytes {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        if u32::read(buffer)? != 0xCAFEBABE {
            return Err(Error::new(ErrorKind::InvalidInput, "Magic bytes do not match 0xCAFEBABE"))
        }
        Ok(ClassMagicBytes)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ClassVersion {
    pub major: u16,
    pub minor: u16,
}

impl Readable for ClassVersion {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        Ok(ClassVersion {
            minor: buffer.read_u16:: < BigEndian>() ?,
            major: buffer.read_u16:: < BigEndian>() ?,
        })
    }
}

impl ClassVersion {
    pub const fn new(major: u16, minor: u16) -> Self {
        ClassVersion { major, minor }
    }
}

impl PartialOrd for ClassVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.major.cmp(&other.major) {
            Ordering::Equal => Some(self.minor.cmp(&other.minor)),
            x => Some(x),
        }
    }
}

impl Ord for ClassVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => self.minor.cmp(&other.minor),
            x => x,
        }
    }
}

// bitflags! {
//     pub struct AccessFlags: u16 {
//         const PUBLIC = 0x0001;
//         const PRIVATE = 0x0002;
//         const PROTECTED = 0x0004;
//         const STATIC = 0x0008;
//         const FINAL = 0x0010;
//         const SUPER = 0x0020;
//         const SYNCHRONIZED = 0x0020;
//         const VOLATILE = 0x0040;
//         const BRIDGE = 0x0040;
//         const TRANSIENT = 0x0080;
//         const VARARGS = 0x0080;
//         const NATIVE = 0x0100;
//         const INTERFACE = 0x0200;
//         const ABSTRACT = 0x0400;
//         const STRICT = 0x0800;
//         const SYNTHETIC = 0x1000;
//         const ANNOTATION = 0x2000;
//         const ENUM = 0x4000;
//         const MODULE = 0x8000;
//     }
// }
//
// impl BufferedRead for AccessFlags {
//     fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
//         match AccessFlags::from_bits(buffer.read_u16::<BigEndian>()?) {
//             Some(v) => Ok(v),
//             None => Err(Error::new(
//                 ErrorKind::Other,
//                 "AccessFlags can not be parsed",
//             )),
//         }
//     }
//
//     fn write<T: Write>(&self, buffer: &mut T) -> io::Result<()> {
//         buffer.write_u16::<BigEndian>(self.bits)
//     }
// }

pub struct PeekedClass {
    pub access_flags: ClassAccessFlags,
    pub this_class: String,
    pub super_class: Option<String>,
}

impl Readable for PeekedClass {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        simple_grammar! {
            struct PartialClass {
                _magic: ClassMagicBytes,
                _version: ClassVersion,
                constants: RawConstantPool,
                access_flags: ClassAccessFlags,
                this_class: u16,
                super_class: u16,
            }
        }

        let peeked = PartialClass::read(buffer)?;

        let this_class = peeked.constants.class_name(peeked.this_class).to_string();

        let super_class = match peeked.super_class {
            0 => None,
            x => Some(peeked.constants.class_name(x).to_string())
        };

        Ok(PeekedClass {
            access_flags: peeked.access_flags,
            this_class,
            super_class,
        })
    }
}


simple_grammar! {
    #[derive(Debug, Clone)]
    pub struct Class {
        _magic: ClassMagicBytes,
        pub version: ClassVersion,
        pub constants: RawConstantPool,
        pub access_flags: ClassAccessFlags,
        pub this_class: u16,
        pub super_class: u16,
        pub interfaces: Vec<u16>,
        pub fields: Vec<FieldInfo>,
        pub methods: Vec<MethodInfo>,
        pub attributes: Vec<AttributeInfo>,
    }
}

impl Class {
    // pub fn bootstrap_methods(&self) -> Option<Vec<BootstrapMethod>> {
    //     let constants = self.constants();
    //     for attribute in &self.attributes {
    //         if constants.text(attribute.name_index) == "BootstrapMethods" {
    //             let mut buffer = Cursor::new(attribute.info.to_owned());
    //             let attr = <Vec<BootstrapMethod> as BufferedRead>::read(&mut buffer).ok()?;
    //             // let class = constants.class_name(attr.class_index).to_owned();
    //             // let (name, desc) = constants.name_and_type(attr.method_index);
    //             return Some(attr);
    //         }
    //     }
    //     None
    // }
    //
    // pub fn enclosing_method(&self) -> Option<(String, String, String)> {
    //     let constants = self.constants();
    //     for attribute in &self.attributes {
    //         if constants.text(attribute.name_index) == "EnclosingMethod" {
    //             let mut buffer = Cursor::new(attribute.info.to_owned());
    //             let attr = EnclosingMethod::read(&mut buffer).ok()?;
    //             let class = constants.class_name(attr.class_index).to_owned();
    //             let (name, desc) = constants.name_and_type(attr.method_index);
    //             return Some((class, name.to_owned(), desc.to_owned()));
    //         }
    //     }
    //     None
    // }
    // pub fn nest_host(&self) -> Option<String> {
    //     let constants = self.constants();
    //     for attribute in &self.attributes {
    //         if constants.text(attribute.name_index) == "NestHost" {
    //             let mut buffer = Cursor::new(attribute.info.to_owned());
    //             let attr = NestHost::read(&mut buffer).ok()?;
    //             return Some(constants.class_name(attr.host_class_index).to_owned());
    //         }
    //     }
    //     None
    // }

    pub fn constants(&self) -> &RawConstantPool {
        &self.constants
        // ConstantPool::from(&self.constants[..])
    }

    // pub fn parse(data: Vec<u8>) -> io::Result<Self> {
    //     let len = data.len();
    //     trace!("A total of {} bytes found!", len);
    //
    //     let mut buffer = Cursor::new(data);
    //
    //     let magic_num = check_magic_number(&mut buffer)?;
    //     trace!("Magic number matches: {}", magic_num);
    //
    //     let class_version = ClassVersion::read(&mut buffer)?;
    //     trace!("Class Version: {:?}", class_version);
    //
    //     let constants = Constant::read_pool(class_version, &mut buffer)?;
    //     trace!("Read {} constant(s)", constants.len());
    //
    //     // for constant in &constants {
    //     //     trace!("\t{:?}", constant);
    //     // }
    //
    //     let access_flags = match AccessFlags::from_bits(buffer.read_u16::<BigEndian>()?) {
    //         Some(v) => v,
    //         None => panic!("Access flags are invalid!"),
    //     };
    //
    //     trace!("Access Flags: {:?}", access_flags);
    //
    //     let this_class = buffer.read_u16::<BigEndian>()?;
    //     let super_class = buffer.read_u16::<BigEndian>()?;
    //
    //     trace!("This class: {:?}", &constants[this_class as usize - 1]);
    //     if super_class != 0 {
    //         trace!("Super class: {:?}", &constants[super_class as usize - 1]);
    //     } else {
    //         trace!("Super class: n/a");
    //     }
    //
    //     let num_interfaces = buffer.read_u16::<BigEndian>()?;
    //     trace!("Num interfaces: {}", num_interfaces);
    //
    //     let mut interfaces = Vec::with_capacity(num_interfaces as usize);
    //     for _ in 0..num_interfaces {
    //         interfaces.push(buffer.read_u16::<BigEndian>()?);
    //     }
    //
    //     let fields = <Vec<FieldInfo>>::read(&mut buffer)?;
    //     trace!("Read {} field(s)", fields.len());
    //
    //     for field in &fields {
    //         trace!("\t{:?}", field);
    //     }
    //
    //     let methods = <Vec<MethodInfo>>::read(&mut buffer)?;
    //     trace!("Read {} method(s)", methods.len());
    //
    //     for method in &methods {
    //         trace!("\t{:?}", method);
    //     }
    //
    //     let attributes = <Vec<AttributeInfo>>::read(&mut buffer)?;
    //     trace!("Read {} attribute(s)", attributes.len());
    //
    //     for attribute in &attributes {
    //         trace!("\t{:?}", attribute);
    //     }
    //
    //     trace!("Read {}/{} bytes!", buffer.position(), len);
    //
    //     Ok(Class {
    //         version: class_version,
    //         constants,
    //         access_flags,
    //         this_class,
    //         super_class,
    //         interfaces,
    //         fields,
    //         methods,
    //         attributes,
    //     })
    // }

    // pub fn peek_name(data: Vec<u8>) -> io::Result<String> {
    //     let mut buffer = Cursor::new(data);
    //     check_magic_number(&mut buffer)?;
    //
    //     let class_version = ClassVersion::read(&mut buffer)?;
    //     let constants = Constant::read_pool(class_version, &mut buffer)?;
    //
    //     let _ = match AccessFlags::from_bits(buffer.read_u16::<BigEndian>()?) {
    //         Some(v) => v,
    //         None => return Err(Error::new(ErrorKind::Other, "Unable to parse access flags")),
    //     };
    //
    //     let this_class = buffer.read_u16::<BigEndian>()?;
    //     if let Constant::Class(ConstantClass { name_index }) = &constants[this_class as usize - 1] {
    //         let index = *name_index;
    //
    //         return match constants[index as usize - 1].expect_utf8() {
    //             Some(v) => Ok(v),
    //             None => Err(Error::new(ErrorKind::Other, "Malformed class constants")),
    //         };
    //     }
    //
    //     Err(Error::new(
    //         ErrorKind::Other,
    //         "Class index does not match constants pool",
    //     ))
    // }

    // pub fn print_method(&self) {
    //     for method in &self.methods {
    //         method.debug_print(&self.constants);
    //     }
    // }

    // pub fn get_dependencies(&self) -> Vec<String> {
    //     let mut dependencies = Vec::new();
    //
    //     for constant in &self.constants {
    //         if let Constant::Class(ConstantClass { name_index }) = constant {
    //             if *name_index == self.this_class {
    //                 continue;
    //             }
    //
    //             let name = self.constants[*name_index as usize - 1]
    //                 .expect_utf8()
    //                 .unwrap();
    //
    //             if name.contains(';') || name.contains('[') {
    //                 let mut buffer = Cursor::new(name.as_bytes().to_vec());
    //                 match FieldDescriptor::read(&mut buffer) {
    //                     Ok(v) => dependencies.extend(v.class_usage()),
    //                     _ => warn!("Unable to parse field descriptor: {:?}", &name),
    //                 };
    //             } else {
    //                 dependencies.push(name);
    //             }
    //         }
    //     }
    //
    //     dependencies
    // }
    //
    // pub fn get_method(&self, name: &str, desc: &str) -> Option<&MethodInfo> {
    //     for method in &self.methods {
    //         if let (Some(a), Some(b)) = (
    //             method.name(&self.constants),
    //             method.descriptor(&self.constants),
    //         ) {
    //             if a == name && b == desc {
    //                 return Some(method);
    //             }
    //         }
    //     }
    //     None
    // }
    //
    // pub fn has_static_field(&self, name: &str) -> bool {
    //     for field in &self.fields {
    //         if let Some(a) = field.name(&self.constants) {
    //             if a == name && field.access.contains(AccessFlags::STATIC) {
    //                 return true;
    //             }
    //         }
    //     }
    //
    //     false
    // }
    //
    // pub fn get_field(&self, name: &str, desc: &str) -> Option<&FieldInfo> {
    //     for field in &self.fields {
    //         if let (Some(a), Some(b)) = (
    //             field.name(&self.constants),
    //             field.descriptor(&self.constants),
    //         ) {
    //             if a == name && b == desc {
    //                 return Some(field);
    //             }
    //         }
    //     }
    //
    //     None
    // }
    //
    pub fn name(&self) -> &str {
        self.constants.class_name(self.this_class)
    }

    pub fn super_class(&self) -> Option<&str> {
        if self.super_class == 0 {
            return None
        }

        Some(self.constants.class_name(self.super_class))

    }

    // pub fn build_object(&self) -> Object {
    //     let mut field_map = HashMap::new();
    //
    //     for field in &self.fields {
    //         let name = field.name(&self.constants).unwrap();
    //         let value = field.field_type(&self.constants).unwrap().initial_local();
    //         field_map.insert(name, value);
    //     }
    //
    //     Object::Instance {
    //         fields: field_map,
    //         class: self.name(),
    //     }
    // }

    // /// Get list of interface classes for checking instanceof
    // pub fn interfaces(&self) -> Vec<String> {
    //     let mut names = Vec::with_capacity(self.interfaces.len());
    //
    //     for index in &self.interfaces {
    //         let class_name = self.constants[*index as usize - 1].expect_class().unwrap();
    //         names.push(
    //             self.constants[class_name as usize - 1]
    //                 .expect_utf8()
    //                 .unwrap(),
    //         );
    //     }
    //
    //     names
    // }
}

simple_grammar! {
    #[derive(Debug, Clone)]
    pub struct FieldInfo {
        pub access: FieldAccessFlags,
        pub name_index: u16,
        pub descriptor_index: u16,
        pub attributes: Vec<AttributeInfo>,
    }
}

// impl FieldInfo {
//     pub fn name(&self, pool: &[Constant]) -> Option<String> {
//         pool[self.name_index as usize - 1].expect_utf8()
//     }
//
//     pub fn descriptor(&self, pool: &[Constant]) -> Option<String> {
//         pool[self.descriptor_index as usize - 1].expect_utf8()
//     }
//
//     pub fn field_type(&self, pool: &[Constant]) -> Option<FieldDescriptor> {
//         let desc = pool[self.descriptor_index as usize - 1].expect_utf8()?;
//         FieldDescriptor::read(&mut Cursor::new(desc.as_bytes().to_vec())).ok()
//     }
// }

// impl BufferedRead for FieldInfo {
//     fn read<T: Read + Seek>(buffer: &mut T) -> io::Result<Self> {
//         Ok(FieldInfo {
//             access: AccessFlags::read(buffer)?,
//             name_index: buffer.read_u16::<BigEndian>()?,
//             descriptor_index: buffer.read_u16::<BigEndian>()?,
//             attributes: <Vec<AttributeInfo>>::read(buffer)?,
//         })
//     }
//
//     fn write<T: Write + Seek>(&self, buffer: &mut T) -> io::Result<()> {
//         buffer.write_u16::<BigEndian>(self.access.bits)?;
//         buffer.write_u16::<BigEndian>(self.name_index)?;
//         buffer.write_u16::<BigEndian>(self.descriptor_index)?;
//         self.attributes.write(buffer)
//     }
// }

// #[derive(Debug, Clone)]
// pub struct AttributeInfo {
//     pub name_index: u16,
//     info: Vec<u8>,
// }
//
// impl DebugWithConst for AttributeInfo {
//     fn fmt(&self, f: &mut Formatter<'_>, pool: &ConstantPool<'_>) -> std::fmt::Result {
//         let mut buffer = Cursor::new(self.info.clone());
//         match pool.text(self.name_index) {
//             "Code" => CodeAttribute::read(&mut buffer).unwrap().fmt(f, pool),
//             "LineNumberTable" => LineNumberTable::read(&mut buffer).unwrap().fmt(f, pool),
//             "BootstrapMethods" => BootstrapMethods::read(&mut buffer).unwrap().fmt(f, pool),
//             "SourceFile" => SourceFile::read(&mut buffer).unwrap().fmt(f, pool),
//             "InnerClasses" => InnerClasses::read(&mut buffer).unwrap().fmt(f, pool),
//             "LocalVariableTable" => LocalVariableTable::read(&mut buffer).unwrap().fmt(f, pool),
//             "Exceptions" => Exceptions::read(&mut buffer).unwrap().fmt(f, pool),
//             "RuntimeVisibleAnnotations" => RuntimeVisibleAnnotations::read(&mut buffer)
//                 .unwrap()
//                 .fmt(f, pool),
//             "Signature" => Signature::read(&mut buffer).unwrap().fmt(f, pool),
//             "StackMapTable" => StackMapTable::read(&mut buffer).unwrap().fmt(f, pool),
//             x => panic!("Unable to decode attribute {} for DebugWithConst", x),
//         }
//     }
// }
//
// impl BufferedRead for AttributeInfo {
//     fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
//         let name_index = buffer.read_u16::<BigEndian>()?;
//         let length = buffer.read_u32::<BigEndian>()?;
//
//         let mut info = vec![0u8; length as usize];
//         buffer.read_exact(&mut info)?;
//
//         Ok(Self { name_index, info })
//     }
//
//     fn write<T: Write>(&self, buffer: &mut T) -> io::Result<()> {
//         buffer.write_u16::<BigEndian>(self.name_index)?;
//         buffer.write_u32::<BigEndian>(self.info.len() as u32)?;
//         buffer.write_all(&self.info)
//     }
// }
simple_grammar!{
    #[derive(Debug, Clone)]
    pub struct MethodInfo {
        access: MethodAccessFlags,
        name_index: u16,
        descriptor_index: u16,
        attributes: Vec<AttributeInfo>,
    }
}

// impl DebugWithConst for MethodInfo {
//     fn fmt(&self, f: &mut Formatter<'_>, pool: &ConstantPool<'_>) -> std::fmt::Result {
//         writeln!(
//             f,
//             "{} {}",
//             pool.text(self.name_index),
//             pool.text(self.descriptor_index)
//         )?;
//         write!(f, "  Access: {:?}", self.access)?;
//
//         if !self.attributes.is_empty() {
//             write!(f, "\n  Attributes:")?;
//             for attr in &self.attributes {
//                 writeln!(f)?;
//                 attr.tabbed_fmt(f, pool, 2)?;
//             }
//         }
//
//         Ok(())
//     }
// }

// impl MethodInfo {
//     pub fn name(&self, pool: &[Constant]) -> Option<String> {
//         pool[self.name_index as usize - 1].expect_utf8()
//     }
//
//     pub fn descriptor(&self, pool: &[Constant]) -> Option<String> {
//         pool[self.descriptor_index as usize - 1].expect_utf8()
//     }
//
//     pub fn code(&self, pool: &ConstantPool) -> CodeAttribute {
//         for attr in &self.attributes {
//             if pool.text(attr.name_index) != "Code" {
//                 continue;
//             }
//
//             let mut buffer = Cursor::new(attr.info.clone());
//             return CodeAttribute::read(&mut buffer).unwrap();
//         }
//         panic!("Unable to find Code attribute in method")
//     }
//
//     pub fn debug_print(&self, pool: &[Constant]) {
//         let name = &pool[self.name_index as usize - 1];
//         let desc = &pool[self.descriptor_index as usize - 1];
//
//         match (name.expect_utf8(), desc.expect_utf8()) {
//             (Some(a), Some(b)) => println!("Method: {} desc: {}", a, b),
//             (a, b) => println!("Method: {:?} desc: {:?}", a, b),
//         }
//
//         for attr in &self.attributes {
//             if let Some(v) = pool[attr.name_index as usize - 1].expect_utf8() {
//                 match v.as_ref() {
//                     "Code" => println!("\tCode:"),
//                     x => {
//                         println!("\t{}: skipped", x);
//                         continue;
//                     }
//                 }
//             }
//
//             print_bytes(&attr.info);
//             let mut buffer = Cursor::new(attr.info.clone());
//             let code_attribute = CodeAttribute::read(&mut buffer).unwrap();
//             println!("\t\t{:?}", code_attribute);
//
//             for attribute in &code_attribute.attributes {
//                 let name = pool[attribute.name_index as usize - 1]
//                     .expect_utf8()
//                     .unwrap();
//                 println!("\t\t{}:", name);
//                 print_bytes(&attribute.info);
//             }
//
//             // for instr in reader.parse(&mut buffer).unwrap() {
//             //     println!("\t\t{:?}", instr);
//             // }
//         }
//     }
// }

// impl BufferedRead for MethodInfo {
//     fn read<T: Read + Seek>(buffer: &mut T) -> io::Result<Self> {
//         Ok(MethodInfo {
//             access: AccessFlags::read(buffer)?,
//             name_index: buffer.read_u16::<BigEndian>()?,
//             descriptor_index: buffer.read_u16::<BigEndian>()?,
//             attributes: <Vec<AttributeInfo>>::read(buffer)?,
//         })
//     }
//
//     fn write<T: Write + Seek>(&self, buffer: &mut T) -> io::Result<()> {
//         self.access.write(buffer)?;
//         buffer.write_u16::<BigEndian>(self.name_index)?;
//         buffer.write_u16::<BigEndian>(self.descriptor_index)?;
//
//         self.attributes.write(buffer)
//     }
// }

