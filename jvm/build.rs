use std::env::var;
use std::fs::File;
use std::io::{self, BufWriter, Write};

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

    for symbol in &symbols {
        println!(
            "cargo:rustc-cdylib-link-arg=-Wl,--defsym={}={}_impl",
            symbol, symbol
        );
    }
}
