use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use hashbrown::{HashMap, HashSet};
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::attribute::CodeAttribute;
use crate::constant_pool::{Constant, ConstantClass, ConstantPool};
use crate::jar::{unpack_jar, Jar, Manifest};
use crate::jvm::mem::FieldDescriptor;
use crate::version::{check_magic_number, ClassVersion};

bitflags! {
    pub struct AccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const SUPER = 0x0020;
        const VOLATILE = 0x0040;
        const TRANSIENT = 0x0080;
        const NATIVE = 0x0100;
        const INTERFACE = 0x0200;
        const ABSTRACT = 0x0400;
        const STRICT = 0x0800;
        const SYNTHETIC = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM = 0x4000;
        const MODULE = 0x8000;
    }
}

pub trait BufferedRead: Sized {
    fn read_str(string: &str) -> io::Result<Self> {
        let mut buffer = Cursor::new(string.as_bytes().to_vec());
        Self::read(&mut buffer)
    }

    fn read(buffer: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        Self::read_versioned(ClassVersion(0, 0), buffer)
    }

    fn write(&self, _: &mut Cursor<&mut Vec<u8>>) -> io::Result<()> {
        unimplemented!("Write has not yet been implemented for this struct!")
    }

    // TODO: deprecate in favor of Vec<T>::read(buffer)
    fn read_vec(buffer: &mut Cursor<Vec<u8>>) -> io::Result<Vec<Self>> {
        let count = buffer.read_u16::<BigEndian>()?;
        let mut vec = Vec::with_capacity(count as usize);

        for _ in 0..count {
            vec.push(Self::read(buffer)?);
        }

        Ok(vec)
    }

    fn read_versioned(_: ClassVersion, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        Self::read(buffer)
    }
}

impl<T: BufferedRead> BufferedRead for Vec<T> {
    fn read(buffer: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        T::read_vec(buffer)
    }

    fn write(&self, buffer: &mut Cursor<&mut Vec<u8>>) -> io::Result<()> {
        buffer.write_u16::<BigEndian>(self.len() as u16)?;

        for value in self {
            value.write(buffer)?;
        }

        Ok(())
    }
}

impl BufferedRead for AccessFlags {
    fn read(buffer: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        match AccessFlags::from_bits(buffer.read_u16::<BigEndian>()?) {
            Some(v) => Ok(v),
            None => Err(Error::new(
                ErrorKind::Other,
                "AccessFlags can not be parsed",
            )),
        }
    }

    fn write(&self, buffer: &mut Cursor<&mut Vec<u8>>) -> io::Result<()> {
        buffer.write_u16::<BigEndian>(self.bits)
    }
}

#[derive(Debug, Clone)]
pub struct Class {
    version: ClassVersion,
    pub constants: Vec<Constant>,
    pub access_flags: AccessFlags,
    this_class: u16,
    super_class: u16,
    interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    methods: Vec<MethodInfo>,
    attributes: Vec<AttributeInfo>,
}

impl Class {
    #[deprecated(since = "0.2.0", note = "Replace with new Class::constants method")]
    pub fn old_constants(&self) -> &[Constant] {
        &self.constants
    }

    pub fn constants(&self) -> ConstantPool {
        ConstantPool::from(&self.constants[..])
    }

    pub fn write(&self) -> io::Result<Vec<u8>> {
        let mut vec = Vec::new();
        let mut buffer = Cursor::new(&mut vec);

        buffer.write_u32::<BigEndian>(0xCAFE_BABE)?;
        self.version.write(&mut buffer)?;
        Constant::write_pool(&self.constants, &mut buffer)?;

        self.access_flags.write(&mut buffer)?;

        buffer.write_u16::<BigEndian>(self.this_class)?;
        buffer.write_u16::<BigEndian>(self.super_class)?;

        buffer.write_u16::<BigEndian>(self.interfaces.len() as u16)?;

        for val in &self.interfaces {
            buffer.write_u16::<BigEndian>(*val)?;
        }

        self.fields.write(&mut buffer)?;
        self.methods.write(&mut buffer)?;
        self.attributes.write(&mut buffer)?;

        Ok(vec)
    }

    pub fn parse(data: Vec<u8>) -> io::Result<Self> {
        let len = data.len();
        trace!("A total of {} bytes found!", len);

        let mut buffer = Cursor::new(data);

        let magic_num = check_magic_number(&mut buffer)?;
        trace!("Magic number matches: {}", magic_num);

        let class_version = ClassVersion::read(&mut buffer)?;
        trace!("Class Version: {:?}", class_version);

        let constants = Constant::read_pool(class_version, &mut buffer)?;
        trace!("Read {} constant(s)", constants.len());

        // for constant in &constants {
        //     trace!("\t{:?}", constant);
        // }

        let access_flags = match AccessFlags::from_bits(buffer.read_u16::<BigEndian>()?) {
            Some(v) => v,
            None => panic!("Access flags are invalid!"),
        };

        trace!("Access Flags: {:?}", access_flags);

        let this_class = buffer.read_u16::<BigEndian>()?;
        let super_class = buffer.read_u16::<BigEndian>()?;

        trace!("This class: {:?}", &constants[this_class as usize - 1]);
        if super_class != 0 {
            trace!("Super class: {:?}", &constants[super_class as usize - 1]);
        } else {
            trace!("Super class: n/a");
        }

        let num_interfaces = buffer.read_u16::<BigEndian>()?;
        trace!("Num interfaces: {}", num_interfaces);

        let mut interfaces = Vec::with_capacity(num_interfaces as usize);
        for _ in 0..num_interfaces {
            interfaces.push(buffer.read_u16::<BigEndian>()?);
        }

        let fields = FieldInfo::read_vec(&mut buffer)?;
        trace!("Read {} field(s)", fields.len());

        for field in &fields {
            trace!("\t{:?}", field);
        }

        let methods = MethodInfo::read_vec(&mut buffer)?;
        trace!("Read {} method(s)", methods.len());

        for method in &methods {
            trace!("\t{:?}", method);
        }

        let attributes = AttributeInfo::read_vec(&mut buffer)?;
        trace!("Read {} attribute(s)", attributes.len());

        for attribute in &attributes {
            trace!("\t{:?}", attribute);
        }

        trace!("Read {}/{} bytes!", buffer.position(), len);

        Ok(Class {
            version: class_version,
            constants,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        })
    }

    pub fn peek_name(data: Vec<u8>) -> io::Result<String> {
        let mut buffer = Cursor::new(data);
        check_magic_number(&mut buffer)?;

        let class_version = ClassVersion::read(&mut buffer)?;
        let constants = Constant::read_pool(class_version, &mut buffer)?;

        let _ = match AccessFlags::from_bits(buffer.read_u16::<BigEndian>()?) {
            Some(v) => v,
            None => return Err(Error::new(ErrorKind::Other, "Unable to parse access flags")),
        };

        let this_class = buffer.read_u16::<BigEndian>()?;
        if let Constant::Class(ConstantClass { name_index }) = &constants[this_class as usize - 1] {
            let index = *name_index;

            return match constants[index as usize - 1].expect_utf8() {
                Some(v) => Ok(v),
                None => Err(Error::new(ErrorKind::Other, "Malformed class constants")),
            };
        }

        Err(Error::new(
            ErrorKind::Other,
            "Class index does not match constants pool",
        ))
    }

    pub fn print_method(&self) {
        for method in &self.methods {
            method.debug_print(&self.constants);
        }
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        let mut dependencies = Vec::new();

        for constant in &self.constants {
            if let Constant::Class(ConstantClass { name_index }) = constant {
                if *name_index == self.this_class {
                    continue;
                }

                let name = self.constants[*name_index as usize - 1]
                    .expect_utf8()
                    .unwrap();

                if name.contains(';') || name.contains('[') {
                    let mut buffer = Cursor::new(name.as_bytes().to_vec());
                    match FieldDescriptor::read(&mut buffer) {
                        Ok(v) => dependencies.extend(v.class_usage()),
                        _ => warn!("Unable to parse field descriptor: {:?}", &name),
                    };
                } else {
                    dependencies.push(name);
                }
            }
        }

        dependencies
    }

    pub fn get_method(&self, name: &str, desc: &str) -> Option<&MethodInfo> {
        for method in &self.methods {
            if let (Some(a), Some(b)) = (
                method.name(&self.constants),
                method.descriptor(&self.constants),
            ) {
                if a == name && b == desc {
                    return Some(method);
                }
            }
        }
        None
    }

    pub fn get_field(&self, name: &str, desc: &str) -> Option<&FieldInfo> {
        for field in &self.fields {
            if let (Some(a), Some(b)) = (
                field.name(&self.constants),
                field.descriptor(&self.constants),
            ) {
                if a == name && b == desc {
                    return Some(field);
                }
            }
        }

        None
    }

    pub fn name(&self) -> String {
        let name_idx = self.constants[self.this_class as usize - 1]
            .expect_class()
            .unwrap();
        self.constants[name_idx as usize - 1].expect_utf8().unwrap()
    }

    pub fn super_class(&self) -> String {
        let name_idx = self.constants[self.super_class as usize - 1]
            .expect_class()
            .unwrap();
        self.constants[name_idx as usize - 1].expect_utf8().unwrap()
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

    /// Get list of interface classes for checking instanceof
    pub fn interfaces(&self) -> Vec<String> {
        let mut names = Vec::with_capacity(self.interfaces.len());

        for index in &self.interfaces {
            let class_name = self.constants[*index as usize - 1].expect_class().unwrap();
            names.push(
                self.constants[class_name as usize - 1]
                    .expect_utf8()
                    .unwrap(),
            );
        }

        names
    }
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub access: AccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>,
}

impl FieldInfo {
    pub fn name(&self, pool: &[Constant]) -> Option<String> {
        pool[self.name_index as usize - 1].expect_utf8()
    }

    pub fn descriptor(&self, pool: &[Constant]) -> Option<String> {
        pool[self.descriptor_index as usize - 1].expect_utf8()
    }

    pub fn field_type(&self, pool: &[Constant]) -> Option<FieldDescriptor> {
        let desc = pool[self.descriptor_index as usize - 1].expect_utf8()?;
        FieldDescriptor::read(&mut Cursor::new(desc.as_bytes().to_vec())).ok()
    }
}

impl BufferedRead for FieldInfo {
    fn read(buffer: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        Ok(FieldInfo {
            access: AccessFlags::read(buffer)?,
            name_index: buffer.read_u16::<BigEndian>()?,
            descriptor_index: buffer.read_u16::<BigEndian>()?,
            attributes: AttributeInfo::read_vec(buffer)?,
        })
    }

    fn write(&self, buffer: &mut Cursor<&mut Vec<u8>>) -> io::Result<()> {
        buffer.write_u16::<BigEndian>(self.access.bits)?;
        buffer.write_u16::<BigEndian>(self.name_index)?;
        buffer.write_u16::<BigEndian>(self.descriptor_index)?;
        self.attributes.write(buffer)
    }
}

#[derive(Debug, Clone)]
pub struct AttributeInfo {
    name_index: u16,
    info: Vec<u8>,
}

impl BufferedRead for AttributeInfo {
    fn read(buffer: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        let name_index = buffer.read_u16::<BigEndian>()?;
        let length = buffer.read_u32::<BigEndian>()?;

        let mut info = vec![0u8; length as usize];
        buffer.read_exact(&mut info)?;

        Ok(Self { name_index, info })
    }

    fn write(&self, buffer: &mut Cursor<&mut Vec<u8>>) -> io::Result<()> {
        buffer.write_u16::<BigEndian>(self.name_index)?;
        buffer.write_u32::<BigEndian>(self.info.len() as u32)?;
        buffer.write_all(&self.info)
    }
}

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub access: AccessFlags,
    name_index: u16,
    descriptor_index: u16,
    attributes: Vec<AttributeInfo>,
}

impl MethodInfo {
    pub fn name(&self, pool: &[Constant]) -> Option<String> {
        pool[self.name_index as usize - 1].expect_utf8()
    }

    pub fn descriptor(&self, pool: &[Constant]) -> Option<String> {
        pool[self.descriptor_index as usize - 1].expect_utf8()
    }

    pub fn code(&self, pool: &[Constant]) -> CodeAttribute {
        for attr in &self.attributes {
            if let Some(v) = pool[attr.name_index as usize - 1].expect_utf8() {
                if &v != "Code" {
                    continue;
                }
            }

            let mut buffer = Cursor::new(attr.info.clone());
            return CodeAttribute::read(&mut buffer).unwrap();
        }
        panic!("Unable to find Code attribute in method")
    }

    pub fn debug_print(&self, pool: &[Constant]) {
        let name = &pool[self.name_index as usize - 1];
        let desc = &pool[self.descriptor_index as usize - 1];

        match (name.expect_utf8(), desc.expect_utf8()) {
            (Some(a), Some(b)) => println!("Method: {} desc: {}", a, b),
            (a, b) => println!("Method: {:?} desc: {:?}", a, b),
        }

        for attr in &self.attributes {
            if let Some(v) = pool[attr.name_index as usize - 1].expect_utf8() {
                match v.as_ref() {
                    "Code" => println!("\tCode:"),
                    x => {
                        println!("\t{}: skipped", x);
                        continue;
                    }
                }
            }

            print_bytes(&attr.info);
            let mut buffer = Cursor::new(attr.info.clone());
            let code_attribute = CodeAttribute::read(&mut buffer).unwrap();
            println!("\t\t{:?}", code_attribute);

            for attribute in &code_attribute.attributes {
                let name = pool[attribute.name_index as usize - 1]
                    .expect_utf8()
                    .unwrap();
                println!("\t\t{}:", name);
                print_bytes(&attribute.info);
            }

            // for instr in reader.parse(&mut buffer).unwrap() {
            //     println!("\t\t{:?}", instr);
            // }
        }
    }
}

impl BufferedRead for MethodInfo {
    fn read(buffer: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        Ok(MethodInfo {
            access: AccessFlags::read(buffer)?,
            name_index: buffer.read_u16::<BigEndian>()?,
            descriptor_index: buffer.read_u16::<BigEndian>()?,
            attributes: AttributeInfo::read_vec(buffer)?,
        })
    }

    fn write(&self, buffer: &mut Cursor<&mut Vec<u8>>) -> io::Result<()> {
        self.access.write(buffer)?;
        buffer.write_u16::<BigEndian>(self.name_index)?;
        buffer.write_u16::<BigEndian>(self.descriptor_index)?;

        self.attributes.write(buffer)
    }
}

pub fn print_bytes(buffer: &[u8]) {
    let mut idx = 0;

    for byte in buffer {
        if idx % 8 == 0 {
            print!("\t\t");
        }
        print!("{:02x} ", byte);

        idx += 1;

        if idx != 0 && idx % 8 == 0 {
            println!();
        }
    }
    if idx % 8 != 0 {
        println!();
    }
}

#[derive(Debug, Default)]
struct UnpackedJar {
    dir: PathBuf,
    manifest: Manifest,
}

#[derive(Default, Debug)]
pub struct ClassLoader {
    loaded: HashMap<String, Class>,
    class_path: ClassPath,
    load_requests: HashSet<String>,
    loaded_jars: HashMap<PathBuf, UnpackedJar>,
}

impl ClassLoader {
    pub fn from_class_path(class_path: ClassPath) -> Self {
        ClassLoader {
            loaded: Default::default(),
            class_path,
            load_requests: Default::default(),
            loaded_jars: Default::default(),
        }
    }

    fn read_file(path: &PathBuf) -> io::Result<Vec<u8>> {
        let mut file = File::open(path)?;

        // Use seek to get length of file
        let length = file.seek(SeekFrom::End(0))?;
        file.seek(SeekFrom::Start(0))?;

        // Read file to vec
        let mut data = Vec::with_capacity(length as usize);
        file.read_to_end(&mut data)?;

        Ok(data)
    }

    pub fn load_new(&mut self, path: &PathBuf) -> io::Result<()> {
        let data = ClassLoader::read_file(path)?;
        let class = Class::parse(data)?;
        let class_name_index = match &class.constants[class.this_class as usize - 1] {
            Constant::Class(class) => class.name_index,
            _ => return Err(Error::new(ErrorKind::Other, "Class name not found!")),
        };

        let class_name = match &class.constants[class_name_index as usize - 1].expect_utf8() {
            Some(v) => v.clone(),
            None => return Err(Error::new(ErrorKind::Other, "Class name not found!")),
        };

        debug!("Loaded Class {} from {:?}", &class_name, path);
        self.loaded.insert(class_name, class);
        Ok(())
    }

    pub fn unpack_jar(&mut self, file: &PathBuf) -> io::Result<()> {
        info!("Unpacking jar {} for reading", file.display());
        let unpack_folder = unpack_jar(file)?;
        let mut jar = Jar::new(unpack_folder.clone())?;

        // TODO: It would be better if we picked the manifest for our java version in a multi-jar
        for meta in &mut jar.meta {
            let mut manifest = meta.read_manifest()?;
            manifest.check_entries(&unpack_folder).unwrap();
            manifest.verify_entries(&unpack_folder).unwrap();

            self.loaded_jars.insert(
                file.clone(),
                UnpackedJar {
                    dir: unpack_folder.clone(),
                    manifest,
                },
            );
            // break;
        }

        Ok(())
    }

    pub fn preload_class_path(&mut self) -> io::Result<bool> {
        let changes = self.class_path.preload_search_path()?;
        info!("Preloaded {} classes", self.class_path.found_classes.len());
        Ok(changes)
    }

    pub fn class(&self, name: &str) -> Option<&Class> {
        self.loaded.get(name)
    }

    pub fn is_loaded(&self, name: &str) -> bool {
        self.loaded.contains_key(name)
    }

    pub fn attempt_load(&mut self, class: &str) -> io::Result<bool> {
        if self.loaded.contains_key(class) {
            return Ok(true);
        }

        // debug!("Attempting to load class {}", class);
        let ret = match self.class_path.found_classes.get(class) {
            Some(v) => {
                // Annoyingly we need to clone this so we can make a second mutable reference to self
                let load_path = v.clone();

                // Gonna have to take the long approach
                if load_path.extension().and_then(OsStr::to_str) == Some("jar") {
                    // check if jar has already been unpacked
                    if !self.loaded_jars.contains_key(&load_path) {
                        self.unpack_jar(&load_path)?;
                    }

                    let unpacked = self.loaded_jars.get(&load_path).unwrap();
                    let unpacked = unpacked.dir.join(format!("{}.class", class));
                    self.load_new(&unpacked)?
                } else {
                    // Just a regular class so we can just load it normally
                    self.load_new(&load_path)?;
                }

                self.load_requests.remove(class);
                Ok(true)
            }
            None => {
                error!("Unable to find class {} in class path!", class);
                // let mut timeout = 20;
                // for entry in self.class_path.found_classes.keys() {
                //     debug!("Class entry: {}", entry);
                //     timeout -= 1;
                //     if timeout == 0 {
                //         break
                //     }
                // }

                Ok(false)
            }
        };

        if class != "java/lang/Object" {
            let super_class = self.loaded.get(class).unwrap().super_class();
            self.attempt_load(&super_class)?;
        }

        ret
    }

    pub fn load_dependents(&mut self, class: &str) -> io::Result<()> {
        let mut touched = HashSet::new();
        let mut queue = Vec::new();
        queue.push(class.to_string());

        while let Some(target) = queue.pop() {
            if touched.contains(&target) {
                continue;
            }

            if !self.loaded.contains_key(&target) {
                self.attempt_load(&target)?;
            }

            queue.extend(self.loaded.get(&target).unwrap().get_dependencies());
            touched.insert(target);
        }

        Ok(())
    }

    pub fn class_path(&self) -> &ClassPath {
        &self.class_path
    }
}

#[derive(Debug, Clone)]
pub struct ClassPath {
    java_home: PathBuf,
    search_path: Vec<PathBuf>,
    found_classes: HashMap<String, PathBuf>,
}

impl ClassPath {
    pub fn new(java_dir: Option<PathBuf>, class_path: Option<Vec<PathBuf>>) -> io::Result<Self> {
        let lib = match java_dir {
            Some(path) => match ClassPath::check_lib_for_rt(&path) {
                Some(v) => v,
                None => {
                    warn!("Unable to use provided JAVA_HOME: must have rt.jar");
                    ClassPath::find_rt_dir()?
                }
            },
            None => ClassPath::find_rt_dir()?,
        };
        info!("Found Java lib: {}", lib.display());

        let mut search_path = match class_path {
            Some(mut path) => {
                path.drain_filter(|x| !x.exists())
                    .for_each(|x| warn!("Unable to find {}; dropped from classpath", x.display()));
                path
            }
            None => Vec::new(),
        };

        let java_home = lib.parent().unwrap().to_path_buf();
        search_path.insert(0, lib);
        info!("Loaded class path:");
        for entry in &search_path {
            info!("\t{}", entry.display());
        }

        Ok(Self {
            java_home,
            search_path,
            found_classes: HashMap::new(),
        })
    }

    pub fn java_home(&self) -> &PathBuf {
        &self.java_home
    }

    pub fn find_rt_dir() -> io::Result<PathBuf> {
        info!("Searching for valid Java installation");

        // Check java home first
        if let Ok(java_home) = env::var("JAVA_HOME") {
            info!("Found JAVA_HOME: {:?}", &java_home);
            let path = PathBuf::from(&java_home);
            if let Some(path_buf) = ClassPath::check_lib_for_rt(&path) {
                return Ok(path_buf);
            }

            // Check for other versions of java installed in the same folder
            if let Some(parent) = path.parent() {
                for entry in parent.read_dir()? {
                    let alternate_version = entry?.path();

                    if let Some(path_buf) = ClassPath::check_lib_for_rt(&alternate_version) {
                        return Ok(path_buf);
                    }
                }
            }
        } else {
            warn!("Unable to find JAVA_HOME");
        }

        if cfg!(windows) {
            if let Ok(v) = ClassPath::search_dir_for_rt(PathBuf::from("C:\\Program Files\\Java")) {
                return Ok(v)
            }

            ClassPath::search_dir_for_rt(PathBuf::from("C:\\Program Files (x86)\\Java"))
        } else if cfg!(unix) {
            ClassPath::search_dir_for_rt(PathBuf::from("/usr/lib/jvm"))
        } else {
            warn!("Unknown platform! Unsure where to search for java installation!");
            Err(Error::new(ErrorKind::Other, "Unable to find rt.jar"))
        }
    }

    pub fn search_dir_for_rt(search_dir: PathBuf) -> io::Result<PathBuf> {
        info!(
            "Searching for java installation in {}",
            search_dir.display()
        );

        for entry in search_dir.read_dir()? {
            let alternate_version = entry?.path();

            if let Some(path_buf) = ClassPath::check_lib_for_rt(&alternate_version) {
                return Ok(path_buf);
            }
        }

        warn!("Unable to find a compatible java installation! Try installing any JRE or JDK8 or lower");
        Err(Error::new(ErrorKind::Other, "Unable to find rt.jar"))
    }

        fn check_lib_for_rt(path: &PathBuf) -> Option<PathBuf> {
        let jdk_lib = path.join("jre/lib/rt.jar");
        if jdk_lib.exists() && jdk_lib.is_file() {
            return Some(jdk_lib.parent()?.to_path_buf());
        }

        let jre_lib = path.join("lib/rt.jar");
        if jre_lib.exists() && jre_lib.is_file() {
            return Some(jre_lib.parent()?.to_path_buf());
        }

        if path.join("lib").exists() {
            info!(
                "Found Java Installation {}, but does not contain rt.jar",
                path.display()
            );
        }

        None
    }

    pub fn preload_search_path(&mut self) -> io::Result<bool> {
        let mut changes = false;

        for path in &self.search_path.clone() {
            if path.is_dir() {
                changes |= self.preload_dir(path)?;
            } else if path.is_file() && path.extension().and_then(OsStr::to_str) == Some("jar") {
                changes |= self.preload_jar(path)?;
            } else if path.is_file() && path.extension().and_then(OsStr::to_str) == Some("class") {
                changes |= self.preload_class(path)?;
            } else {
                warn!(
                    "Unable to interpret {} while loading class path",
                    path.display()
                );
            }
        }

        Ok(changes)
    }

    pub fn preload_class(&mut self, file: &PathBuf) -> io::Result<bool> {
        debug!("Preloading class: {}", file.display());

        if !file.is_file() || file.extension().and_then(OsStr::to_str) != Some("class") {
            return Err(Error::new(
                ErrorKind::Other,
                format!("File {} is not a valid class", file.display()),
            ));
        }

        let data = ClassLoader::read_file(file)?;
        let name = Class::peek_name(data)?;

        if !self.found_classes.contains_key(&name) {
            self.found_classes.insert(name, file.clone());
            return Ok(true);
        }

        info!(
            "Ignored class {} since version already exists in class path",
            file.display()
        );
        Ok(false)
    }

    pub fn preload_jar(&mut self, file: &PathBuf) -> io::Result<bool> {
        debug!("Preloading jar: {}", file.display());

        let mut jar = ZipArchive::new(File::open(file)?)?;
        let mut changes = false;

        for i in 0..jar.len() {
            if let Some(path) = jar.by_index(i)?.enclosed_name() {
                if let Some(name) = path.to_str() {
                    let filtered_name = match name.strip_suffix(".class") {
                        Some(v) => v,
                        None => name,
                    };

                    if !self.found_classes.contains_key(filtered_name) {
                        self.found_classes
                            .insert(filtered_name.to_string(), file.clone());
                        changes = true;
                    }
                }
            }
        }

        Ok(changes)
    }

    pub fn preload_dir(&mut self, file: &PathBuf) -> io::Result<bool> {
        debug!("Preloading directory: {}", file.display());
        let mut changes = false;

        for entry in WalkDir::new(file) {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            // info!("Visiting {}", path.display());
            if path.extension().and_then(OsStr::to_str) == Some("jar") {
                changes |= self.preload_jar(&path.to_path_buf())?;
            } else if path.extension().and_then(OsStr::to_str) == Some("class") {
                changes |= self.preload_class(&path.to_path_buf())?;
            }
        }

        Ok(changes)
    }

    pub fn load_classes(&mut self) {}
}

impl Default for ClassPath {
    fn default() -> Self {
        ClassPath::new(None, None).unwrap()
    }
}
