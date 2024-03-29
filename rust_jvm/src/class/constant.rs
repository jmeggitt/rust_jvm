use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::io::{self, Cursor, Error, ErrorKind, Read, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num_traits::FromPrimitive;

use crate::class::version::ClassVersion;
use crate::class::{BufferedRead, DebugWithConst};
use crate::jvm::mem::FieldDescriptor;
use crate::util::ThinArcSlice;
use std::ops::{Deref, Index};

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ConstantPool {
    // TODO: Maybe switch back to Arc<[Constant]> and accept fat pointers on operand stack
    pool: ThinArcSlice<Constant>,
}

impl ConstantPool {
    pub fn into_raw(self) -> *const Constant {
        self.pool.into_raw()
    }

    /// Get a constant pool from a raw pointer.
    ///
    /// # Safety
    /// The function should only be called at most one time for each value provided from
    /// [ConstantPool::into_raw]. Using pointers from other sources will lead to undefined behavior.
    pub unsafe fn from_raw(ptr: *const Constant) -> Self {
        ConstantPool {
            pool: ThinArcSlice::from_raw(ptr),
        }
    }
}

impl Default for ConstantPool {
    fn default() -> Self {
        ConstantPool {
            pool: ThinArcSlice::from(Vec::new()),
        }
    }
}

impl From<Vec<Constant>> for ConstantPool {
    fn from(value: Vec<Constant>) -> Self {
        ConstantPool {
            pool: ThinArcSlice::from(value),
        }
    }
}

impl Deref for ConstantPool {
    type Target = [Constant];

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

// TODO: Finish adding helper functions and use in main system
impl ConstantPool {
    pub fn text(&self, index: u16) -> &str {
        match &self[index] {
            Constant::Utf8(ConstantUtf8Info { text }) => text.as_ref(),
            x => panic!("Expected Utf8 constant, but found {:?}", x),
        }
    }

    pub fn class_name(&self, index: u16) -> &str {
        match &self[index] {
            Constant::Class(ConstantClass { name_index }) => self.text(*name_index),
            x => panic!("Expected Class constant, but found {:?}", x),
        }
    }

    pub fn name_and_type(&self, index: u16) -> (&str, &str) {
        match &self[index] {
            Constant::NameAndType(v) => {
                let ConstantNameAndType {
                    name_index,
                    descriptor_index,
                } = v;
                (self.text(*name_index), self.text(*descriptor_index))
            }
            x => panic!("Expected NameAndType constant, but found {:?}", x),
        }
    }

    // TODO: Deprecate in favor of class_element_desc?
    pub fn class_element_ref(&self, index: u16) -> (&str, &str, &str) {
        let (class_index, name_and_type) = match &self[index] {
            Constant::FieldRef(v) => {
                let ConstantFieldRef {
                    class_index,
                    name_and_type_index,
                } = v;
                (*class_index, *name_and_type_index)
            }
            Constant::MethodRef(v) => {
                let ConstantMethodRef {
                    class_index,
                    name_and_type_index,
                } = v;
                (*class_index, *name_and_type_index)
            }
            Constant::InterfaceMethodRef(v) => {
                let ConstantInterfaceMethodRef {
                    class_index,
                    name_and_type_index,
                } = v;
                (*class_index, *name_and_type_index)
            }
            x => panic!(
                "Expected FieldRef/MethodRef/InterfaceMethodRef constant, but found {:?}",
                x
            ),
        };

        let (name, desc) = self.name_and_type(name_and_type);
        (self.class_name(class_index), name, desc)
    }

    pub fn class_element_desc(&self, index: u16) -> ClassElement {
        let (class, element, desc) = self.class_element_ref(index);
        ClassElement::new(class, element, desc)
    }
}

impl Index<u16> for ConstantPool {
    type Output = Constant;

    fn index(&self, index: u16) -> &Self::Output {
        usize::from(index)
            .checked_sub(1)
            .and_then(|index| self.pool.get(index))
            .expect("index is a valid position in constant pool")
    }
}

#[derive(Clone)]
pub struct ClassElement {
    pub class: String,
    pub element: String,
    pub desc: String,
}

impl ClassElement {
    pub fn new<S: ToString>(class: S, element: S, desc: S) -> Self {
        ClassElement {
            class: class.to_string(),
            element: element.to_string(),
            desc: desc.to_string(),
        }
    }

    pub fn build_desc(&self) -> FieldDescriptor {
        match FieldDescriptor::read_str(&self.desc) {
            Ok(v) => v,
            Err(e) => panic!("Expected FieldDescriptor: {:?}", e),
        }
    }
}

impl Debug for ClassElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{} {}", &self.class, &self.element, &self.desc)
    }
}

#[derive(Debug, Clone)]
pub enum Constant {
    Utf8(ConstantUtf8Info),
    Int(ConstantInteger),
    Float(ConstantFloat),
    Long(ConstantLong),
    Double(ConstantDouble),
    Class(ConstantClass),
    String(ConstantString),
    FieldRef(ConstantFieldRef),
    MethodRef(ConstantMethodRef),
    InterfaceMethodRef(ConstantInterfaceMethodRef),
    NameAndType(ConstantNameAndType),
    MethodHandle(ConstantMethodHandle),
    MethodType(ConstantMethodType),
    Dynamic(ConstantDynamic),
    InvokeDynamic(ConstantInvokeDynamic),
    Module(ConstantModule),
    Package(ConstantPackage),

    // Due to a poor choice in the JVM specification, 8 byte constants must take up 2 slots
    // for indexing.
    Placeholder,
}

impl DebugWithConst for Constant {
    fn fmt(&self, f: &mut Formatter<'_>, pool: &ConstantPool) -> std::fmt::Result {
        match self {
            Constant::Utf8(ConstantUtf8Info { text }) => write!(f, "Utf8({:?})", text),
            Constant::Int(ConstantInteger { value }) => write!(f, "Int({:?})", value),
            Constant::Float(ConstantFloat { value }) => write!(f, "Float({:?})", value),
            Constant::Long(ConstantLong { value }) => write!(f, "Long({:?})", value),
            Constant::Double(ConstantDouble { value }) => write!(f, "Double({:?})", value),
            Constant::Class(ConstantClass { name_index }) => {
                write!(f, "Class({})", pool.text(*name_index))
            }
            Constant::String(ConstantString { string_index }) => {
                write!(f, "String({:?})", pool.text(*string_index))
            }
            Constant::FieldRef(ConstantFieldRef {
                class_index,
                name_and_type_index,
            }) => {
                let (name, r#type) = pool.name_and_type(*name_and_type_index);
                write!(
                    f,
                    "FieldRef({}::{} {})",
                    pool.text(*class_index),
                    name,
                    r#type
                )
            }
            Constant::MethodRef(ConstantMethodRef {
                class_index,
                name_and_type_index,
            }) => {
                let (name, r#type) = pool.name_and_type(*name_and_type_index);
                write!(
                    f,
                    "MethodRef({}::{} {})",
                    pool.text(*class_index),
                    name,
                    r#type
                )
            }
            Constant::InterfaceMethodRef(ConstantInterfaceMethodRef {
                class_index,
                name_and_type_index,
            }) => {
                let (name, r#type) = pool.name_and_type(*name_and_type_index);
                write!(
                    f,
                    "InterfaceMethodRef({}::{} {})",
                    pool.text(*class_index),
                    name,
                    r#type
                )
            }
            Constant::NameAndType(ConstantNameAndType {
                name_index,
                descriptor_index,
            }) => write!(
                f,
                "NameAndType({} {})",
                pool.text(*name_index),
                pool.text(*descriptor_index)
            ),
            Constant::MethodHandle(ConstantMethodHandle {
                reference_kind,
                index,
            }) => {
                let (class, name, r#type) = pool.class_element_ref(*index);
                write!(
                    f,
                    "MethodHandle({:?} {}::{} {})",
                    reference_kind, class, name, r#type
                )
            }
            Constant::MethodType(ConstantMethodType { descriptor_index }) => {
                write!(f, "MethodType({})", pool.text(*descriptor_index))
            }
            Constant::Dynamic(ConstantDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            }) => {
                let (name, r#type) = pool.name_and_type(*name_and_type_index);
                write!(
                    f,
                    "Dynamic(attr[{}], {} {})",
                    bootstrap_method_attr_index, name, r#type
                )
            }
            Constant::InvokeDynamic(ConstantInvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            }) => {
                let (name, r#type) = pool.name_and_type(*name_and_type_index);
                write!(
                    f,
                    "InvokeDynamic(attr[{}], {} {})",
                    bootstrap_method_attr_index, name, r#type
                )
            }
            Constant::Module(ConstantModule { name_index }) => {
                write!(f, "Module({})", pool.text(*name_index))
            }
            Constant::Package(ConstantPackage { name_index }) => {
                write!(f, "Package({})", pool.text(*name_index))
            }
            Constant::Placeholder => write!(f, "Placeholder"),
        }
    }
}

impl Constant {
    pub fn expect_utf8(&self) -> Option<String> {
        match self {
            Constant::Utf8(ConstantUtf8Info { text }) => Some(text.clone()),
            _ => None,
        }
    }

    pub fn expect_class(&self) -> Option<u16> {
        match self {
            Constant::Class(ConstantClass { name_index }) => Some(*name_index),
            _ => None,
        }
    }

    pub fn expect_name_and_type(&self) -> Option<ConstantNameAndType> {
        match self {
            Constant::NameAndType(x) => Some(*x),
            _ => None,
        }
    }

    pub fn read_pool(version: ClassVersion, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Vec<Self>> {
        let count = buffer.read_u16::<BigEndian>()?;
        let mut vec = Vec::with_capacity(count as usize);

        let mut index = 1;

        while index < count {
            let val = Self::read_versioned(version, buffer)?;
            trace!("\t{}/{}: {:?}", index, count, &val);

            match &val {
                Constant::Long(..) | Constant::Double(..) => {
                    vec.push(val);
                    vec.push(Constant::Placeholder);
                    index += 2;
                }
                _ => {
                    vec.push(val);
                    index += 1;
                }
            };
        }

        Ok(vec)
    }

    pub fn write_pool(pool: &[Constant], buffer: &mut Cursor<&mut Vec<u8>>) -> io::Result<()> {
        buffer.write_u16::<BigEndian>(pool.len() as u16 + 1)?;

        for constant in pool {
            constant.write(buffer)?;
        }

        Ok(())
    }

    fn read_versioned(version: ClassVersion, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        Ok(match buffer.read_u8()? {
            ConstantUtf8Info::TAG => {
                Constant::Utf8(ConstantUtf8Info::attempt_read(version, buffer)?)
            }
            ConstantInteger::TAG => Constant::Int(ConstantInteger::attempt_read(version, buffer)?),
            ConstantFloat::TAG => Constant::Float(ConstantFloat::attempt_read(version, buffer)?),
            ConstantLong::TAG => Constant::Long(ConstantLong::attempt_read(version, buffer)?),
            ConstantDouble::TAG => Constant::Double(ConstantDouble::attempt_read(version, buffer)?),
            ConstantClass::TAG => Constant::Class(ConstantClass::attempt_read(version, buffer)?),
            ConstantString::TAG => Constant::String(ConstantString::attempt_read(version, buffer)?),
            ConstantFieldRef::TAG => {
                Constant::FieldRef(ConstantFieldRef::attempt_read(version, buffer)?)
            }
            ConstantMethodRef::TAG => {
                Constant::MethodRef(ConstantMethodRef::attempt_read(version, buffer)?)
            }
            ConstantInterfaceMethodRef::TAG => Constant::InterfaceMethodRef(
                ConstantInterfaceMethodRef::attempt_read(version, buffer)?,
            ),
            ConstantNameAndType::TAG => {
                Constant::NameAndType(ConstantNameAndType::attempt_read(version, buffer)?)
            }
            ConstantMethodHandle::TAG => {
                Constant::MethodHandle(ConstantMethodHandle::attempt_read(version, buffer)?)
            }
            ConstantMethodType::TAG => {
                Constant::MethodType(ConstantMethodType::attempt_read(version, buffer)?)
            }
            ConstantDynamic::TAG => {
                Constant::Dynamic(ConstantDynamic::attempt_read(version, buffer)?)
            }
            ConstantInvokeDynamic::TAG => {
                Constant::InvokeDynamic(ConstantInvokeDynamic::attempt_read(version, buffer)?)
            }
            ConstantModule::TAG => Constant::Module(ConstantModule::attempt_read(version, buffer)?),
            ConstantPackage::TAG => {
                Constant::Package(ConstantPackage::attempt_read(version, buffer)?)
            }
            x => panic!("Unknown tag: {}", x),
        })
    }

    fn write(&self, buffer: &mut Cursor<&mut Vec<u8>>) -> io::Result<()> {
        match self {
            Constant::Utf8(v) => v.tagged_write(buffer),
            Constant::Int(v) => v.tagged_write(buffer),
            Constant::Float(v) => v.tagged_write(buffer),
            Constant::Long(v) => v.tagged_write(buffer),
            Constant::Double(v) => v.tagged_write(buffer),
            Constant::Class(v) => v.tagged_write(buffer),
            Constant::String(v) => v.tagged_write(buffer),
            Constant::FieldRef(v) => v.tagged_write(buffer),
            Constant::MethodRef(v) => v.tagged_write(buffer),
            Constant::InterfaceMethodRef(v) => v.tagged_write(buffer),
            Constant::NameAndType(v) => v.tagged_write(buffer),
            Constant::MethodHandle(v) => v.tagged_write(buffer),
            Constant::MethodType(v) => v.tagged_write(buffer),
            Constant::Dynamic(v) => v.tagged_write(buffer),
            Constant::InvokeDynamic(v) => v.tagged_write(buffer),
            Constant::Module(v) => v.tagged_write(buffer),
            Constant::Package(v) => v.tagged_write(buffer),
            Constant::Placeholder => Ok(()),
        }
    }
}

pub trait ConstantPoolTag: Sized + Debug + BufferedRead {
    /// Used to facilitate parsing
    const TAG: u8;

    /// Class version this tag was added
    const MIN_VERSION: ClassVersion = ClassVersion(0, 0);

    /// If this constant can be loaded directly to stack. Maybe, if this is a final field?
    const STACK_LOADABLE: Option<ClassVersion> = None;

    fn attempt_read<T: Read + Seek>(version: ClassVersion, buffer: &mut T) -> io::Result<Self> {
        if version.cmp(&Self::MIN_VERSION) == Ordering::Greater {
            return Err(Error::new(
                ErrorKind::Other,
                "Constant pool tag version exceeded class version",
            ));
        }

        Self::read(buffer)
    }

    fn tagged_write<T: Write + Seek>(&self, buffer: &mut T) -> io::Result<()> {
        buffer.write_u8(Self::TAG)?;
        self.write(buffer)
    }
}

readable_struct! {
    pub struct ConstantClass {
        name_index: u16,
    }
}

impl ConstantPoolTag for ConstantClass {
    const TAG: u8 = 7;
    const MIN_VERSION: ClassVersion = ClassVersion(45, 3);
    const STACK_LOADABLE: Option<ClassVersion> = Some(ClassVersion(49, 0));
}

readable_struct! {
    pub struct ConstantFieldRef {
        class_index: u16,
        name_and_type_index: u16,
    }
}

impl ConstantPoolTag for ConstantFieldRef {
    const TAG: u8 = 9;
    const MIN_VERSION: ClassVersion = ClassVersion(45, 3);
}

readable_struct! {
    pub struct ConstantMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    }
}

impl ConstantPoolTag for ConstantMethodRef {
    const TAG: u8 = 10;
    const MIN_VERSION: ClassVersion = ClassVersion(45, 3);
}

readable_struct! {
    pub struct ConstantInterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    }
}

impl ConstantPoolTag for ConstantInterfaceMethodRef {
    const TAG: u8 = 11;
    const MIN_VERSION: ClassVersion = ClassVersion(45, 3);
}

readable_struct! {
    pub struct ConstantString {
        string_index: u16,
    }
}

impl ConstantPoolTag for ConstantString {
    const TAG: u8 = 8;
    const MIN_VERSION: ClassVersion = ClassVersion(45, 3);
    const STACK_LOADABLE: Option<ClassVersion> = Some(ClassVersion(45, 3));
}

readable_struct! {
    pub struct ConstantInteger {
        value: i32,
    }
}

impl ConstantPoolTag for ConstantInteger {
    const TAG: u8 = 3;
    const MIN_VERSION: ClassVersion = ClassVersion(45, 3);
    const STACK_LOADABLE: Option<ClassVersion> = Some(ClassVersion(45, 3));
}

readable_struct! {
    pub struct ConstantFloat {
        value: f32,
    }
}

impl ConstantPoolTag for ConstantFloat {
    const TAG: u8 = 4;
    const MIN_VERSION: ClassVersion = ClassVersion(45, 3);
    const STACK_LOADABLE: Option<ClassVersion> = Some(ClassVersion(45, 3));
}

readable_struct! {
    pub struct ConstantLong {
        value: i64,
    }
}

impl ConstantPoolTag for ConstantLong {
    const TAG: u8 = 5;
    const MIN_VERSION: ClassVersion = ClassVersion(45, 3);
    const STACK_LOADABLE: Option<ClassVersion> = Some(ClassVersion(45, 3));
}
readable_struct! {
    pub struct ConstantDouble {
        value: f64,
    }
}

impl ConstantPoolTag for ConstantDouble {
    const TAG: u8 = 6;
    const MIN_VERSION: ClassVersion = ClassVersion(45, 3);
    const STACK_LOADABLE: Option<ClassVersion> = Some(ClassVersion(45, 3));
}

// Both fields represent indexes in the same table to CONSTANT_Utf8_info
readable_struct! {
    pub struct ConstantNameAndType {
        name_index: u16,
        descriptor_index: u16,
    }
}

impl ConstantPoolTag for ConstantNameAndType {
    const TAG: u8 = 12;
    const MIN_VERSION: ClassVersion = ClassVersion(45, 3);
}

#[derive(Debug, Clone)]
pub struct ConstantUtf8Info {
    text: String,
}

impl ConstantUtf8Info {
    pub fn decode(buffer: &[u8]) -> Vec<u8> {
        let mut idx = 0;
        let mut result = Vec::with_capacity(buffer.len());

        while idx < buffer.len() {
            if buffer[idx] & 0b1000_0000 == 0 {
                result.push(buffer[idx]);
                idx += 1;
            } else if buffer[idx] & 0b1110_0000 == 0b1100_0000 {
                if buffer[idx] == 0b1100_0000 {
                    result.push(0);
                } else {
                    result.extend_from_slice(&buffer[idx..idx + 2]);
                }
                idx += 2;
            } else if buffer[idx] & 0b1111_0000 == 0b1110_0000 {
                result.extend_from_slice(&buffer[idx..idx + 3]);
                idx += 3;
            } else if buffer[idx] == 0b11101101 {
                let mut code_point: u32 = 0x10000;
                code_point += (buffer[idx + 1] as u32 & 0x0f) << 16;
                code_point += (buffer[idx + 2] as u32 & 0x3f) << 10;
                code_point += (buffer[idx + 3] as u32 & 0x0f) << 6;
                code_point += buffer[idx + 4] as u32 & 0x3f;

                let character = match core::char::from_u32(code_point) {
                    Some(v) => v,
                    None => panic!("Is this even possible?"),
                };

                let mut formatting = [0u8; 4];
                let len = char::encode_utf8(character, &mut formatting).len();
                result.extend_from_slice(&formatting[..len]);
            } else {
                panic!(
                    "Unable to decode constant string with first byte of {}",
                    buffer[idx]
                )
            }
        }

        result.shrink_to_fit();
        result
    }
}

impl ConstantPoolTag for ConstantUtf8Info {
    const TAG: u8 = 1;
    const MIN_VERSION: ClassVersion = ClassVersion::new(45, 3);
}

impl BufferedRead for ConstantUtf8Info {
    fn read<T: Read>(buffer: &mut T) -> io::Result<Self> {
        let len = buffer.read_u16::<BigEndian>()?;

        let mut text_buffer = vec![0u8; len as usize];
        buffer.read_exact(&mut text_buffer)?;

        // TODO: Use proper encoding
        // Ok(ConstantUtf8Info {
        //     text: match String::from_utf8(ConstantUtf8Info::decode(&text_buffer)) {
        //         Ok(v) => v,
        //         Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        //     },
        // })
        Ok(ConstantUtf8Info {
            text: String::from_utf8_lossy(&text_buffer).to_string(),
        })
    }

    // TODO: This is not compliant, but its way faster and works for most common unicode characters
    fn write<T: Write>(&self, buffer: &mut T) -> io::Result<()> {
        buffer.write_u16::<BigEndian>(self.text.len() as u16)?;
        buffer.write_all(self.text.as_bytes())?;
        Ok(())
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive)]
pub enum ReferenceKind {
    GetField = 1,
    GetStatic = 2,
    PutField = 3,
    PutStatic = 4,
    InvokeVirtual = 5,
    InvokeStatic = 6,
    InvokeSpecial = 7,
    NewInvokeSpecial = 8,
    InvokeInterface = 9,
}

impl BufferedRead for ReferenceKind {
    fn read<T: Read + Seek>(buffer: &mut T) -> io::Result<Self> {
        match Self::from_u8(buffer.read_u8()?) {
            Some(v) => Ok(v),
            None => Err(Error::new(
                ErrorKind::Other,
                "Reference kind value out of bounds!",
            )),
        }
    }

    fn write<T: Write + Seek>(&self, buffer: &mut T) -> io::Result<()> {
        buffer.write_u8(*self as u8)
    }
}

readable_struct! {
    pub struct ConstantMethodHandle {
        reference_kind: ReferenceKind,
        index: u16,
    }
}

impl ConstantPoolTag for ConstantMethodHandle {
    const TAG: u8 = 15;
    const MIN_VERSION: ClassVersion = ClassVersion(51, 0);
    const STACK_LOADABLE: Option<ClassVersion> = Some(ClassVersion(51, 0));
}

readable_struct! {
    pub struct ConstantMethodType {
        descriptor_index: u16,
    }
}

impl ConstantPoolTag for ConstantMethodType {
    const TAG: u8 = 16;
    const MIN_VERSION: ClassVersion = ClassVersion(51, 0);
    const STACK_LOADABLE: Option<ClassVersion> = Some(ClassVersion(51, 0));
}

readable_struct! {
    pub struct ConstantDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    }
}

impl ConstantPoolTag for ConstantDynamic {
    const TAG: u8 = 17;
    const MIN_VERSION: ClassVersion = ClassVersion(55, 0);
    const STACK_LOADABLE: Option<ClassVersion> = Some(ClassVersion(55, 0));
}

readable_struct! {
    pub struct ConstantInvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    }
}

impl ConstantPoolTag for ConstantInvokeDynamic {
    const TAG: u8 = 18;
    const MIN_VERSION: ClassVersion = ClassVersion(51, 0);
}

readable_struct! {
    pub struct ConstantModule {
        name_index: u16,
    }
}

impl ConstantPoolTag for ConstantModule {
    const TAG: u8 = 19;
    const MIN_VERSION: ClassVersion = ClassVersion(53, 0);
}

readable_struct! {
    pub struct ConstantPackage {
        name_index: u16,
    }
}

impl ConstantPoolTag for ConstantPackage {
    const TAG: u8 = 20;
    const MIN_VERSION: ClassVersion = ClassVersion(53, 0);
}
