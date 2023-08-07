// use crate::class::class_file::Class;
// use crate::class::constant::Constant;
// use crate::class::jar::{unpack_jar, Jar, Manifest};
use crate::class::Class;
use crate::path::ClassPath;
use crate::read::Readable;
use log::warn;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::{BufReader, Cursor, Error, ErrorKind, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use zip::ZipArchive;
// log_dump!(CLASS_LOADER);

// #[derive(Debug, Default)]
// pub struct UnpackedJar {
//     dir: PathBuf,
//     pub manifest: Manifest,
// }

#[derive(Default, Debug)]
pub struct ClassLoader {
    loaded: HashMap<String, Class>,
    class_path: ClassPath,
    load_requests: HashSet<String>,
    pub loaded_jars: HashMap<PathBuf, ZipArchive<BufReader<File>>>,
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

    pub fn read_file(path: &Path) -> io::Result<Vec<u8>> {
        let mut file = File::open(path)?;

        // Use seek to get length of file
        let length = file.seek(SeekFrom::End(0))?;
        file.seek(SeekFrom::Start(0))?;

        // Read file to vec
        let mut data = Vec::with_capacity(length as usize);
        file.read_to_end(&mut data)?;

        Ok(data)
    }

    pub fn load_from_buffer<T: Read>(&mut self, buffer: &mut T) -> io::Result<()> {
        let class = Class::read(buffer)?;
        // let class_name_index = match &class.constants[class.this_class as usize - 1] {
        //     Constant::Class(class) => class.name_index,
        //     _ => return Err(Error::new(ErrorKind::Other, "Class name not found!")),
        // };
        //
        // let class_name = match &class.constants[class_name_index as usize - 1].expect_utf8() {
        //     Some(v) => v.clone(),
        //     None => return Err(Error::new(ErrorKind::Other, "Class name not found!")),
        // };

        // log_dump!(CLASS_LOADER, "[Explicit Load] {}: {}", &class_name, path.display());
        // log_dump!(CLASS_LOADER, "[Explicit Load]");
        let name = class.name().to_string();

        self.loaded.insert(name, class);
        Ok(())
    }

    // pub fn read_buffer(&mut self, bytes: &[u8]) -> io::Result<()> {
    //     let class = Class::parse(bytes.to_vec())?;
    //     let class_name_index = match &class.constants[class.this_class as usize - 1] {
    //         Constant::Class(class) => class.name_index,
    //         _ => return Err(Error::new(ErrorKind::Other, "Class name not found!")),
    //     };
    //
    //     let class_name = match &class.constants[class_name_index as usize - 1].expect_utf8() {
    //         Some(v) => v.clone(),
    //         None => return Err(Error::new(ErrorKind::Other, "Class name not found!")),
    //     };
    //
    //     // log_dump!(CLASS_LOADER, "[Explicit Load] {}: {}", &class_name, path.display());
    //     // log_dump!(CLASS_LOADER, "[Explicit Load]");
    //     self.loaded.insert(class_name, class);
    //     Ok(())
    // }

    pub fn load_new(&mut self, path: &Path) -> io::Result<()> {
        let mut file = File::open(path)?;
        self.load_from_buffer(&mut file)

        // let data = ClassLoader::read_file(path)?;
        // let class = Class::parse(data)?;
        // let class_name_index = match &class.constants[class.this_class as usize - 1] {
        //     Constant::Class(class) => class.name_index,
        //     _ => return Err(Error::new(ErrorKind::Other, "Class name not found!")),
        // };
        //
        // let class_name = match &class.constants[class_name_index as usize - 1].expect_utf8() {
        //     Some(v) => v.clone(),
        //     None => return Err(Error::new(ErrorKind::Other, "Class name not found!")),
        // };

        // debug!("Loaded Class {} from {:?}", &class_name, path);
        // log_dump!(CLASS_LOADER, "[Explicit Load] {}: {}", &class_name, path.display());
        // log_dump!(CLASS_LOADER, "[Explicit Load]");
        // self.loaded.insert(class_name, class);
        // Ok(())
    }

    // pub fn unpack_jar(&mut self, file: &Path) -> io::Result<()> {
    //     info!("Unpacking jar {} for reading", file.display());
    //     let unpack_folder = unpack_jar(file)?;
    //     let mut jar = Jar::new(unpack_folder.clone())?;
    //
    //     // TODO: It would be better if we picked the manifest for our java version in a multi-jar
    //     for meta in &mut jar.meta {
    //         let mut manifest = meta.read_manifest()?;
    //         manifest.check_entries(&unpack_folder).unwrap();
    //         manifest.verify_entries(&unpack_folder).unwrap();
    //
    //         // self.loaded_jars.insert(
    //         //     file.to_path_buf(),
    //         //     UnpackedJar {
    //         //         dir: unpack_folder.clone(),
    //         //         manifest,
    //         //     },
    //         // );
    //     }
    //
    //     Ok(())
    // }

    // pub fn preload_class_path(&mut self) -> io::Result<bool> {
    //     let changes = self.class_path.preload_search_path()?;
    //     info!("Preloaded {} classes", self.class_path.found_classes.len());
    //     Ok(changes)
    // }

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
                // log_dump!(CLASS_LOADER, "{}: {}", class, v.display());
                // Annoyingly we need to clone this so we can make a second mutable reference to self
                let load_path = v.clone();

                // Gonna have to take the long approach
                if load_path.extension().and_then(OsStr::to_str) == Some("jar") {
                    // check if jar has already been unpacked
                    if !self.loaded_jars.contains_key(&load_path) {
                        // self.unpack_jar(&load_path)?;
                        self.loaded_jars.insert(
                            load_path.clone(),
                            ZipArchive::new(BufReader::new(File::open(&load_path)?))?,
                        );
                    }
                    //
                    let buffer = {
                        let jar = self.loaded_jars.get_mut(&load_path).unwrap();
                        // let unpacked = unpacked.dir.join(format!("{}.class", class));
                        // self.load_new(&unpacked)?

                        // TODO: Hold file descriptor
                        // let mut jar = ZipArchive::new(File::open(&load_path)?)?;

                        let mut entry = match jar.by_name(&format!("{}.class", class)) {
                            Ok(v) => v,
                            Err(e) => {
                                return Err(Error::new(ErrorKind::NotFound, format!("{:?}", e)))
                            }
                        };

                        let mut bytes = Vec::with_capacity(entry.size() as usize);
                        entry.read_to_end(&mut bytes)?;
                        bytes
                    };
                    let mut indexed_buffer = Cursor::new(&buffer);
                    self.load_from_buffer(&mut indexed_buffer)?;
                    // self.read_buffer(&buffer)?;
                } else {
                    // Just a regular class so we can just load it normally
                    self.load_new(&load_path)?;
                }

                self.load_requests.remove(class);
                Ok(true)
            }
            None => {
                if !class.starts_with('[') {
                    warn!("Unable to find class {} in class path!", class);
                }
                if class.contains('.') {
                    panic!("Attempted to find class with '.' in name")
                }
                Ok(false)
            }
        };

        // let super_class = ;
        if let Some(super_class) = self.loaded.get(class).unwrap().super_class() {
            let owned = super_class.to_string();
            self.attempt_load(&owned)?;
        }

        // if class != "java/lang/Object" && matches!(&ret, Ok(true)) {
        //     let super_class = self.loaded.get(class).unwrap().super_class();
        //     self.attempt_load(super_class)?;
        // }

        ret
    }

    // pub fn load_dependents(&mut self, class: &str) -> io::Result<()> {
    //     let mut touched = HashSet::new();
    //     let mut queue = vec![class.to_string()];
    //
    //     while let Some(target) = queue.pop() {
    //         if touched.contains(&target) {
    //             continue;
    //         }
    //
    //         if !self.loaded.contains_key(&target) {
    //             self.attempt_load(&target)?;
    //         }
    //
    //         queue.extend(self.loaded.get(&target).unwrap().get_dependencies());
    //         touched.insert(target);
    //     }
    //
    //     Ok(())
    // }

    // pub fn class_path(&self) -> &ClassPath {
    //     &self.class_path
    // }
}
