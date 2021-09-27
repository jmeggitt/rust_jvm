use crate::class::class_file::Class;
use crate::class::constant::Constant;
use crate::class::jar::{unpack_jar, Jar, Manifest};
use crate::log_dump;
use hashbrown::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::{env, io};
use walkdir::WalkDir;
use zip::ZipArchive;
log_dump!(CLASS_LOADER);

#[derive(Debug, Default)]
pub struct UnpackedJar {
    dir: PathBuf,
    pub manifest: Manifest,
}

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

    fn read_file(path: &Path) -> io::Result<Vec<u8>> {
        let mut file = File::open(path)?;

        // Use seek to get length of file
        let length = file.seek(SeekFrom::End(0))?;
        file.seek(SeekFrom::Start(0))?;

        // Read file to vec
        let mut data = Vec::with_capacity(length as usize);
        file.read_to_end(&mut data)?;

        Ok(data)
    }

    pub fn read_buffer(&mut self, bytes: &[u8]) -> io::Result<()> {
        let class = Class::parse(bytes.to_vec())?;
        let class_name_index = match &class.constants[class.this_class as usize - 1] {
            Constant::Class(class) => class.name_index,
            _ => return Err(Error::new(ErrorKind::Other, "Class name not found!")),
        };

        let class_name = match &class.constants[class_name_index as usize - 1].expect_utf8() {
            Some(v) => v.clone(),
            None => return Err(Error::new(ErrorKind::Other, "Class name not found!")),
        };

        // log_dump!(CLASS_LOADER, "[Explicit Load] {}: {}", &class_name, path.display());
        // log_dump!(CLASS_LOADER, "[Explicit Load]");
        self.loaded.insert(class_name, class);
        Ok(())
    }

    pub fn load_new(&mut self, path: &Path) -> io::Result<()> {
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
        // log_dump!(CLASS_LOADER, "[Explicit Load] {}: {}", &class_name, path.display());
        // log_dump!(CLASS_LOADER, "[Explicit Load]");
        self.loaded.insert(class_name, class);
        Ok(())
    }

    pub fn unpack_jar(&mut self, file: &Path) -> io::Result<()> {
        info!("Unpacking jar {} for reading", file.display());
        let unpack_folder = unpack_jar(file)?;
        let mut jar = Jar::new(unpack_folder.clone())?;

        // TODO: It would be better if we picked the manifest for our java version in a multi-jar
        for meta in &mut jar.meta {
            let mut manifest = meta.read_manifest()?;
            manifest.check_entries(&unpack_folder).unwrap();
            manifest.verify_entries(&unpack_folder).unwrap();

            // self.loaded_jars.insert(
            //     file.to_path_buf(),
            //     UnpackedJar {
            //         dir: unpack_folder.clone(),
            //         manifest,
            //     },
            // );
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
                log_dump!(CLASS_LOADER, "{}: {}", class, v.display());
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
                    self.read_buffer(&buffer)?;
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

        if class != "java/lang/Object" && matches!(&ret, Ok(true)) {
            let super_class = self.loaded.get(class).unwrap().super_class();
            self.attempt_load(&super_class)?;
        }

        ret
    }

    pub fn load_dependents(&mut self, class: &str) -> io::Result<()> {
        let mut touched = HashSet::new();
        let mut queue = vec![class.to_string()];

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
                let mut i = 0;
                while i < path.len() {
                    if !path[i].exists() {
                        warn!(
                            "Unable to find {}; dropped from classpath",
                            path.remove(i).display()
                        );
                    } else {
                        i += 1;
                    }
                }

                // path.drain_filter(|x| !x.exists())
                //     .for_each(|x| warn!("Unable to find {}; dropped from classpath", x.display()));
                path
            }
            None => Vec::new(),
        };

        let java_home = lib.parent().unwrap().to_path_buf();
        search_path.insert(0, lib);
        info!("Loaded class path:");
        log_dump!(CLASS_LOADER, "Loaded class path:");
        for entry in &search_path {
            info!("\t{}", entry.display());
            log_dump!(CLASS_LOADER, "\t{}", entry.display());
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
            log_dump!(CLASS_LOADER, "Found JAVA_HOME: {:?}", &java_home);
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
                return Ok(v);
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

    fn check_lib_for_rt(path: &Path) -> Option<PathBuf> {
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

    pub fn preload_class(&mut self, file: &Path) -> io::Result<bool> {
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
            self.found_classes.insert(name, file.to_path_buf());
            return Ok(true);
        }

        info!(
            "Ignored class {} since version already exists in class path",
            file.display()
        );
        Ok(false)
    }

    pub fn preload_jar(&mut self, file: &Path) -> io::Result<bool> {
        debug!("Preloading jar: {}", file.display());

        let mut jar = ZipArchive::new(BufReader::new(File::open(file)?))?;
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
                            .insert(filtered_name.to_string(), file.to_path_buf());
                        changes = true;
                    }
                }
            }
        }

        Ok(changes)
    }

    pub fn preload_dir(&mut self, file: &Path) -> io::Result<bool> {
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
