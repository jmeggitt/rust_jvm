use walkdir::WalkDir;
use std::io;
use std::process::Command;

pub fn main() {
    println!("cargo:rerun-if-changed=src/**.asm");

    // Because its a pain to compile on windows
    #[cfg(unix)]
    {
        nasm_rs::compile_library("libexec.a", &["src/jvm/exec.asm"]).unwrap();
        // nasm_rs::compile_library_args("libexec.a", &["src/jvm/exec.asm"], &["ineowef"]).unwrap();
        println!("cargo:rustc-link-lib=exec");
    }

    build_stdlib().unwrap();
}

pub fn build_stdlib() -> io::Result<()> {
    println!("cargo:rerun-if-changed=std/jvm/*");

    for entry in WalkDir::new("std/jvm") {
        let entry = entry?;

        if !entry.path().is_file() {
            continue
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
