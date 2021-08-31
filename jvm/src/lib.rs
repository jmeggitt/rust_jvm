// #![feature(drain_filter)]
#![feature(const_type_id)]
// Ensure each result error is either unwrapped or returned
#![deny(unused_must_use)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
#[macro_use]
extern crate num_derive;

pub mod attribute;
pub mod class;
pub mod constant_pool;
pub mod exports;
pub mod instruction;
pub mod jar;
pub mod jvm;
pub mod version;

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
