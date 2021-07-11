use std::io;
use std::process::Command;
use walkdir::WalkDir;

pub fn main() {
    build_stdlib().unwrap();
}

pub fn build_stdlib() -> io::Result<()> {
    println!("cargo:rerun-if-changed=std/jvm/*");

    for entry in WalkDir::new("std/jvm") {
        let entry = entry?;

        if !entry.path().is_file() {
            continue;
        }

        if entry.path().extension() == Some("java".as_ref()) {
            Command::new("javac")
                .arg(entry.path())
                .args(&["-d", "std/out/", "-h", "std/includes/"])
                .status()?;
        }
    }

    Ok(())
}
