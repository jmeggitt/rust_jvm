use std::fs::{create_dir_all, read_dir, File};
use std::io;
use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use zip::ZipArchive;

use crate::read_file;
use hashbrown::HashMap;
use sha2::{Digest, Sha256};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem::replace;
use walkdir::WalkDir;

pub fn unpack_jar<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    let mut jar = ZipArchive::new(File::open(path.as_ref())?)?;

    // TODO: let unpack_dir = tempdir()?;

    let file_name = match path.as_ref().file_name().and_then(|x| x.to_str()) {
        Some(v) => v,
        None => {
            return Err(Error::new(
                ErrorKind::Other,
                "Unable to read jar file name!",
            ))
        }
    };

    // Hash the file name so we don't mix up jars in unpack dir
    let mut hasher = DefaultHasher::new();
    path.as_ref().hash(&mut hasher);
    let file_hash = hasher.finish();

    let unpack_dir;
    if file_name.ends_with(".jar") {
        unpack_dir = PathBuf::from(format!(
            "java_libs/{}-{:0x?}",
            &file_name[..file_name.len() - 4],
            file_hash
        ));
    } else {
        unpack_dir = PathBuf::from(format!("java_libs/{}-{:0x?}", &file_name, file_hash));
    }

    if unpack_dir.exists() {
        return Ok(unpack_dir);
        // return Err(Error::new(ErrorKind::Other, "Directory already exists!"));
    }

    for i in 0..jar.len() {
        let mut file = jar.by_index(i)?;

        let path = match file.enclosed_name() {
            Some(v) => unpack_dir.join(v),
            None => continue,
        };

        trace!("Extracting {:?}", &path);

        if file.name().ends_with("/") {
            create_dir_all(path)?;
        } else {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    create_dir_all(parent)?;
                }
            }

            let mut output = File::create(path)?;
            io::copy(&mut file, &mut output)?;
        }
    }

    Ok(unpack_dir)
}

pub struct Jar {
    pub path: PathBuf,
    pub meta: Vec<MetaInf>,
}

impl Jar {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let mut jar = Jar {
            path,
            meta: Vec::new(),
        };

        jar.load_meta_inf()?;
        Ok(jar)
    }

    pub fn load_meta_inf(&mut self) -> io::Result<()> {
        self.meta.clear();

        let meta_dir = self.path.join("META-INF");
        debug!("Meta dir: {:?}", &meta_dir);

        for entry in read_dir(&meta_dir)? {
            let entry = entry?;

            if !entry.file_type()?.is_dir() {
                continue;
            }

            let file = match entry.file_name().to_str() {
                Some(v) => v.to_string(),
                None => return Err(Error::new(ErrorKind::Other, "Unable to read OsString")),
            };

            match u32::from_str(&file) {
                Ok(v) if v >= 9 => self.meta.push(MetaInf {
                    version: Some(v),
                    path: entry.path(),
                }),
                _ => {}
            }
        }

        if self.meta.is_empty() {
            self.meta.push(MetaInf {
                version: None,
                path: meta_dir,
            });
        }

        Ok(())
    }
}

pub struct LineJoiner<'a, I> {
    iter: &'a mut I,
    previous: Option<String>,
}

impl<'a, I> LineJoiner<'a, I> {
    pub fn new(iter: &'a mut I) -> Self {
        LineJoiner {
            iter,
            previous: None,
        }
    }
}

impl<'a, I: Iterator<Item = io::Result<String>>> Iterator for LineJoiner<'a, I> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (self.iter.next(), &mut self.previous) {
                (Some(Err(e)), _) => return Some(Err(e)),
                (Some(Ok(v)), None) => {
                    if v.len() >= 72 {
                        self.previous = Some(v);
                    } else {
                        return Some(Ok(v));
                    }
                }
                (Some(Ok(v)), Some(pre)) => {
                    if v.len() > 0 && v.as_bytes()[0] == b' ' {
                        pre.push_str(&v[1..]);
                    } else {
                        return Some(Ok(replace(pre, v)));
                    }
                }
                (None, x) if x.is_some() => return Some(Ok(replace(x, None).unwrap())),
                _ => return None,
            }
        }
    }
}

pub struct MetaInf {
    version: Option<u32>,
    pub path: PathBuf,
}

impl MetaInf {
    pub fn read_manifest(&mut self) -> io::Result<Manifest> {
        let manifest_path = self.path.join("MANIFEST.MF");

        let file = File::open(manifest_path)?;
        let reader = BufReader::new(file);

        let mut manifest = Manifest::default();

        let mut jar_entry = None;

        for line in LineJoiner::new(&mut reader.lines()) {
            let line = line?;

            if line.is_empty() {
                continue;
            }

            let (attr, value) = match line.split_once(": ") {
                Some((a, b)) => (a.trim(), b.trim()),
                None => panic!("Unable to split: {:?}", line),
            };

            // FIXME: Numbers are parsed lazily and errors might be ignored
            match attr {
                "Manifest-Version" => {
                    manifest.version = match f32::from_str(value) {
                        Ok(v) => v,
                        Err(e) => return Err(Error::new(ErrorKind::Other, e)),
                    }
                }
                "Created-By" => manifest.creator = Some(value.to_string()),
                "Signature-Version" => manifest.signature_version = u16::from_str(value).ok(),
                "Class-Path" => manifest
                    .class_path
                    .extend(value.split(" ").map(|x| x.to_owned())),
                "Automatic-Module-Name" => unimplemented!("Does not support Automatic-Module-Name"),
                "Multi-Release" => {} // TODO: What does this effect? We already split for multi release
                "Main-Class" => manifest.main_class = Some(value.to_string()),
                "Launcher-Agent-Class" => manifest.launcher_agent_class = Some(value.to_string()),
                "Name" => {
                    if let Some(v) = jar_entry.replace(JarEntry::new(value.to_string())) {
                        manifest.entries.push(v);
                    }
                }
                "Java-Bean" => {
                    jar_entry.get_or_insert_with(Default::default).java_bean =
                        Some(value.eq_ignore_ascii_case("true"))
                }
                "Magic" => {
                    jar_entry.get_or_insert_with(Default::default).magic = Some(value.to_string())
                }
                x if x.contains("-Digest") => {
                    let (algorithm, suffix) = x.split_once("-Digest").unwrap();

                    let hash = match base64::decode(value) {
                        Ok(v) => v,
                        Err(e) => return Err(Error::new(ErrorKind::Other, e)),
                    };

                    let digest = if suffix.len() > 1 {
                        DigestInfo {
                            digest_type: algorithm.to_string(),
                            language: Some(suffix[1..].to_string()),
                            value: hash,
                        }
                    } else {
                        DigestInfo {
                            digest_type: algorithm.to_string(),
                            language: None,
                            value: hash,
                        }
                    };

                    jar_entry.get_or_insert_with(Default::default).digest = Some(digest);
                }
                x => warn!("Unknown manifest value: {:?}", x),
            }
        }

        Ok(manifest)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Manifest {
    version: f32,
    creator: Option<String>,
    signature_version: Option<u16>,
    class_path: Vec<String>,
    main_class: Option<String>,
    launcher_agent_class: Option<String>,
    entries: Vec<JarEntry>,
}

impl Manifest {
    pub fn check_entries(&mut self, path: &PathBuf) -> io::Result<()> {
        let mut entries = HashMap::with_capacity(self.entries.len());

        for entry in self.entries.iter() {
            entries.insert(entry.name.clone(), entry.clone());
        }

        let exclude = path.join("META-INF");

        for entry in WalkDir::new(path) {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.starts_with(&exclude) || entry_path.is_dir() {
                continue;
            }

            let file_path = match entry_path.strip_prefix(path) {
                Ok(v) => v,
                Err(e) => return Err(Error::new(ErrorKind::Other, e)),
            };
            let path_str = format!("{}", file_path.display());

            if !entries.contains_key(&path_str) {
                let new_entry = JarEntry::new(path_str.clone());
                entries.insert(path_str, new_entry);
            }
        }

        self.entries = entries.into_iter().map(|(_, x)| x).collect();
        Ok(())
    }

    pub fn verify_entries(&self, path: &PathBuf) -> io::Result<()> {
        for entry in &self.entries {
            if let Some(digest) = &entry.digest {
                digest.verify(path.join(&entry.name))?;
            }
        }

        info!("Verified hashes of all entries in {:?}", path);
        Ok(())
    }

    pub fn classes(&self, path: &PathBuf) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        for entry in &self.entries {
            if entry.name.ends_with(".class") {
                paths.push(path.join(&entry.name));
            }
        }

        paths
    }
}

#[derive(Default, Debug, Clone)]
pub struct JarEntry {
    name: String,
    content_type: Option<String>,
    java_bean: Option<bool>,
    digest: Option<DigestInfo>,
    magic: Option<String>,
}

impl JarEntry {
    pub fn new(name: String) -> Self {
        JarEntry {
            name,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct DigestInfo {
    digest_type: String,
    language: Option<String>,
    value: Vec<u8>,
}

impl DigestInfo {
    pub fn verify(&self, path: PathBuf) -> io::Result<()> {
        // println!("Verifying digest of {:?}", &path);

        if self.digest_type != "SHA-256" {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Digest {:?}, not supported!", &self.digest_type),
            ));
        }

        if self.language.is_some() {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Digest language {:?}, not supported!", &self.language),
            ));
        }

        // TODO: Find better way to read file to hasher
        let mut hasher = Sha256::new();
        let data = read_file(path.to_str().unwrap())?;
        hasher.write_all(&data)?;

        let result = hasher.finalize();

        if result[..] != self.value[..] {
            return Err(Error::new(ErrorKind::Other, "Hash did not match!"));
        }

        Ok(())
    }
}
