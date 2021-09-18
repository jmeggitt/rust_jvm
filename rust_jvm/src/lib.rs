#![feature(c_variadic)]
// Ensure each result error is either unwrapped or returned
#![deny(unused_must_use)]

#[macro_use]
#[cfg(feature = "thread_profiler")]
extern crate thread_profiler;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
#[macro_use]
extern crate num_derive;

#[macro_use]
pub mod exports;

// pub mod attribute;
// pub mod r#mod;
pub mod class;
// pub mod constant_pool;
pub mod instruction;
// pub mod jar;
pub mod jvm;
// pub mod version;

#[macro_export]
macro_rules! profile_scope_cfg {
    ($($arg:tt)*) => {
        #[cfg(feature = "thread_profiler")]
        thread_profiler::ProfileScope::new(format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_dump {
    ($name:ident) => {
        lazy_static::lazy_static!{
            pub static ref $name: parking_lot::Mutex<std::io::BufWriter<std::fs::File>> = {
                parking_lot::Mutex::new(std::io::BufWriter::new(std::fs::File::create(format!("{}.dump", stringify!($name).to_lowercase())).unwrap()))
            };
        }
    };
    ($name:ident, $($tokens:tt)+) => {
        if !cfg!(feature = "quiet") {
            use std::io::Write;
            writeln!(&mut *$name.lock(), $($tokens)+).unwrap();
            $name.lock().flush().unwrap();
        }
    };
}

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
pub fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;

    // Use seek to get length of file
    let length = file.seek(SeekFrom::End(0))?;
    file.seek(SeekFrom::Start(0))?;

    // Read file to vec
    let mut data = Vec::with_capacity(length as usize);
    file.read_to_end(&mut data)?;

    Ok(data)
}
