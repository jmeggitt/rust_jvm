#![feature(str_split_once)]
#![feature(drain_filter)]
// #![feature(asm)]
#![feature(const_type_id)]
// Ensure each result error is either unwrapped or returned
#![deny(unused_must_use)]
// TODO: Remove later
#![allow(dead_code)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
#[macro_use]
extern crate num_derive;

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

use log::LevelFilter;
use pretty_env_logger::env_logger::Target;

use crate::class::{ClassLoader, ClassPath};
use crate::jvm::JVM;

mod attribute;
mod class;
mod constant_pool;
mod instruction;
mod jar;
mod jvm;
mod types;
mod version;

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

fn main() {
    pretty_env_logger::formatted_builder()
        .target(Target::Stdout)
        .filter_level(LevelFilter::Debug)
        .init();
    // pretty_env_logger::init();
    // Builder::new()
    //     .target(Target::Stdout)
    //     .filter_level(LevelFilter::Debug)
    //     .init();

    info!("Starting...");

    // let data = read_file("21w19a/com/mojang/blaze3d/systems/RenderSystem.class").expect("Unable to read file!");
    // let data = read_file("21w19a/aca.class").expect("Unable to read file!");
    // let data = read_file("Simple.class").expect("Unable to read file!");
    // let clone = data.clone();

    // let class = Class::parse(data).unwrap();
    // let write = class.write().unwrap();

    // panic!();
    // let jar_file = jar::unpack_jar("rt.jar").unwrap();
    // let jar_file = jar::unpack_jar("21w19a.jar").unwrap();
    // info!("Jar File: {}", jar_file.display());

    // let mut jar = jar::Jar::new(jar_file.clone()).unwrap();

    let class_path = ClassPath::new(None, Some(vec!["std/out".into()])).unwrap();
    let mut class_loader = ClassLoader::from_class_path(class_path);
    class_loader.preload_class_path().unwrap();

    class_loader.load_new(&"Simple.class".into()).unwrap();

    let mut jvm = JVM::new(class_loader);

    let print_class = jvm::Object::build_class(&mut jvm, "jvm/hooks/PrintStreamHook");
    println!("{:?}", print_class);

    // jvm.entry_point("Simple", Vec::new()).unwrap();

    // class.build_simplified_constants();
}
