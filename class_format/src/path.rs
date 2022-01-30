use std::collections::{HashMap, HashSet};
use std::{env, io};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, Cursor, Error, ErrorKind, Read, Seek};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::ZipArchive;
use log::{warn, info, trace, debug};
use zip::result::{ZipError, ZipResult};
use crate::loader::ClassLoader;
use crate::class::{Class, PeekedClass};
use crate::read::Readable;

pub struct DynamicClassPath {
    java_home: PathBuf,
    search_path: Vec<PathBuf>,
    resolved_paths: Vec<ResolvedPath>,
    search_progress: usize,
}

enum ResolvedPath {
    ClassFile {
        name: String,
        path: PathBuf,
    },
    JarFile {
        packages: HashSet<String>,
        buffer: ZipArchive<BufReader<File>>,
    },
}

impl ResolvedPath {

    #[inline]
    fn try_read_class(&mut self, target: &str) -> io::Result<Option<Class>> {
        match self {
            ResolvedPath::ClassFile { name, path } => {
                if target == name {
                    let mut file = BufReader::new(File::open(path)?);
                    return Ok(Some(Class::read(&mut file)?))
                }

                Ok(None)
            }
            ResolvedPath::JarFile { packages, buffer } => {
                if let Some(index) = target.find('/') {
                    if !packages.contains(&target[..index]) {
                        return Ok(None)
                    }
                }

                match buffer.by_name(&format!("{}.class", target)) {
                    Ok(mut file_buffer) => Ok(Some(Class::read(&mut file_buffer)?)),
                    Err(ZipError::Io(e)) => Err(e),
                    Err(ZipError::FileNotFound) => Ok(None),
                    Err(e) => Err(Error::new(ErrorKind::Other, e)),
                }
            }
        }
    }

    fn for_class(base: &Path, file: &Path) -> io::Result<ResolvedPath> {
        match file.strip_prefix(base) {
            Ok(v) => Ok(ResolvedPath::ClassFile {
                name: format!("{}", v.display()),
                path: file.to_path_buf(),
            }),
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
        }
    }

    fn for_jar(file: &Path) -> io::Result<ResolvedPath> {
        let archive = ZipArchive::new(BufReader::new(File::open(&file)?))?;
        let mut file_names = HashSet::new();

        for path in archive.file_names() {
            if let Some(idx) = path.find('/') {
                // Check with contains first to avoid copy
                if !file_names.contains(&path[..idx]) {
                    file_names.insert(path[..idx].to_string());
                }
            }
        }

        Ok(ResolvedPath::JarFile {
            packages: Default::default(),
            // path: Default::default(),
            buffer: archive,
        })
    }
}

impl DynamicClassPath {
    /// For a given class name, resolve the location of the class and return an anonymous trait
    /// implementing Read and Seek that can be used to get the bytes of the requested class.
    pub fn resolve_class(&mut self, target: &str) -> io::Result<Class> {
        for resolved in &mut self.resolved_paths {
            if let Some(class) = resolved.try_read_class(target)? {
                return Ok(class)
            }
        }

        while self.search_progress < self.search_path.len() {
            let prev_idx = self.resolved_paths.len();
            self.resolve_next_search_location();

            for resolved in &mut self.resolved_paths[prev_idx..] {
                if let Some(class) = resolved.try_read_class(target)? {
                    return Ok(class)
                }
            }
        }

        Err(Error::new(ErrorKind::NotFound, target.to_string()))
    }


    fn resolve_next_search_location(&mut self) -> io::Result<()> {
        if self.search_progress >= self.search_path.len() {
            return Ok(());
        }

        self.search_progress += 1;
        let root_path = self.search_path[self.search_progress - 1].to_path_buf();

        if !root_path.exists() {
            return Err(Error::new(ErrorKind::NotFound, root_path.display().to_string()));
        }

        if root_path.is_file() {
            self.resolve_file(&root_path, &root_path)?;
        } else {
            for entry in WalkDir::new(&root_path) {
                self.resolve_file(root_path.as_ref(), entry?.path())?;

            }
        }

        Ok(())
    }

    #[inline]
    fn resolve_file<P: AsRef<Path>>(&mut self, base: P, target: P) -> io::Result<()> {
        match target.as_ref().extension().and_then(OsStr::to_str) {
            Some("jar") | Some("zip") => self.resolved_paths.push(ResolvedPath::for_jar(target.as_ref())?),
            Some("class") => self.resolved_paths.push(ResolvedPath::for_class(base.as_ref(), target.as_ref())?),
            _ => {}
        }

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct ClassPath {
    java_home: PathBuf,
    search_path: Vec<PathBuf>,
    pub(crate) found_classes: HashMap<String, PathBuf>,
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
        // log_dump!(CLASS_LOADER, "Loaded class path:");
        for entry in &search_path {
            info!("\t{}", entry.display());
            // log_dump!(CLASS_LOADER, "\t{}", entry.display());
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
            // log_dump!(CLASS_LOADER, "Found JAVA_HOME: {:?}", &java_home);
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

        // let data = ClassLoader::read_file(file)?
        let mut data = BufReader::new(File::open(file)?);
        let peeked = PeekedClass::read(&mut data)?;
        // let name = Class::peek_name(data)?;

        if !self.found_classes.contains_key(&peeked.this_class) {
            self.found_classes.insert(peeked.this_class, file.to_path_buf());
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

pub fn find_jdk_installation() {}
