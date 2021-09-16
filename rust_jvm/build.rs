use std::env::var;
use std::fs::{create_dir, File};
use std::io::{self, BufWriter, Write};
use std::process::{Command, Stdio};
use walkdir::WalkDir;

const VERSION_NODE: &str = "SUNWprivate_1.1";

fn read_symbols() -> Vec<String> {
    let input = String::from_utf8_lossy(include_bytes!("jvm_symbols.in"));
    let mut symbols = Vec::new();

    for line in input.split('\n') {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        symbols.push(line.to_string());
    }

    symbols
}

fn write_symbol_list(path: &str, symbols: &[String]) -> io::Result<()> {
    let mut script = BufWriter::new(File::create(path)?);

    writeln!(script, "{} {{", VERSION_NODE)?;
    writeln!(script, "global:")?;

    for symbol in symbols {
        writeln!(script, "{};", symbol)?;
    }

    writeln!(script, "local:")?;

    for symbol in symbols {
        writeln!(script, "{}_impl;", symbol)?;
    }

    writeln!(script, "}};")?;
    Ok(())
}

fn main() {
    build_stdlib().unwrap();
    println!("cargo:rerun-if-changed=jvm_symbols.in");
    println!("cargo:rustc-cdylib-link-arg=-fuse-ld=lld");

    let out_dir = var("OUT_DIR").unwrap();
    let symbol_list_path = out_dir + "/version_list";
    println!(
        "cargo:rustc-cdylib-link-arg=-Wl,--version-script={}",
        &symbol_list_path
    );

    let symbols = read_symbols();
    write_symbol_list(&symbol_list_path, &symbols).unwrap();

    // if cfg!(windows) {
    //     println!("cargo:rustc-cdylib-link-arg=/export:_GetStringUTFLength@8=_GetStringUTFLength");
    // }

    for symbol in &symbols {
        if cfg!(windows) {
            println!(
                "cargo:rustc-cdylib-link-arg=/export:{}={}_impl",
                symbol, symbol
            );
        } else {
            println!(
                "cargo:rustc-cdylib-link-arg=-Wl,--defsym={}={}_impl",
                symbol, symbol
            );
        }
    }

    // Since Rust does not support implementing variadic functions, a tiny amount of c is needed to redirect the functions to a supported abi
    println!("cargo:rerun-if-changed=src/va_link_support.c");
    cc::Build::new()
        .file("src/va_link_support.c")
        .compile("va_link_support");

    // Define linkage for symbols expected on windows by other java 8 dlls that are not present in regular java 8
    def_sym("JVM_BeforeHalt", "JVM_Unsupported");
    def_sym("JVM_CopySwapMemory", "JVM_Unsupported");
    def_sym("JVM_SetVmMemoryPressure", "JVM_Unsupported");
    def_sym("JVM_GetVmMemoryPressure", "JVM_Unsupported");
    def_sym("JVM_GetManagementExt", "JVM_Unsupported");
    def_sym("JVM_KnownToNotExist", "JVM_Unsupported");
    def_sym("JVM_GetResourceLookupCacheURLs", "JVM_Unsupported");
    def_sym("JVM_GetResourceLookupCache", "JVM_Unsupported");
    def_sym("JVM_GetTemporaryDirectory", "JVM_Unsupported");
}

pub fn def_sym(name: &str, backing: &str) {
    if cfg!(windows) {
        println!("cargo:rustc-cdylib-link-arg=/export:{}={}", name, backing);
    } else {
        println!(
            "cargo:rustc-cdylib-link-arg=-Wl,--defsym={}={}",
            name, backing
        );
    }
}

pub fn build_stdlib() -> io::Result<()> {
    println!("cargo:rerun-if-changed=java/*");
    println!("cargo:rerun-if-changed=rust_jvm/java/*");
    let target = format!("{}/java_std", var("OUT_DIR").unwrap());
    let _ = create_dir(&target).ok();

    for entry in WalkDir::new("java") {
        let entry = entry?;

        if !entry.path().is_file() {
            continue;
        }

        if entry.path().extension() == Some("java".as_ref()) {
            run(Command::new("javac")
                .arg(entry.path())
                .args(&["-d", &target]))
            .unwrap();
        }
    }

    Ok(())
}

fn run(cmd: &mut Command) -> Result<(), String> {
    println!("running: {:?}", cmd);

    let status = match cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
    {
        Ok(status) => status,

        Err(e) => return Err(format!("failed to spawn process: {}", e)),
    };

    if !status.success() {
        return Err(format!("nonzero exit status: {}", status));
    }
    Ok(())
}
