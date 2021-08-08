use pretty_env_logger::formatted_builder;
use pretty_env_logger::env_logger::Target;
use log::LevelFilter;
use std::env::var;
use glob::glob;
use log::{error, info};

mod args;
use args::*;

use jvm::class::{ClassPath, ClassLoader};
use jvm::jvm::JavaEnv;
use std::path::PathBuf;
use std::process::exit;

fn main() {
    let mut opts = ManualOpts::default()
        .arg(ArgHandler {
            name: "verbose",
            // Technically, java only uses -verbose, but -v was available and is more standard
            aliases: vec!["-v", "-verbose"],
            arg_type: ArgType::Flag,
        })
        .arg(ArgHandler {
            name: "class_path",
            aliases: vec!["--class-path", "-classpath", "-cp"],
            arg_type: ArgType::Valued,
        })
        .arg(ArgHandler {
            name: "jar",
            aliases: vec!["-jar"],
            arg_type: ArgType::Flag,
        })
        .parse();

    println!("{:?}", &opts);

    // First setup logging so we can see future errors in verbose mode
    let log_level = match opts.has_flag("verbose") {
        true => LevelFilter::Debug,
        false => LevelFilter::Error,
    };

    formatted_builder()
        .target(Target::Stdout)
        .filter_level(log_level)
        .init();

    if opts.has_flag("verbose") {
        info!("Arguments: {:?}", get_java_args());
        info!("Running in verbose mode");
    }

    // Class path separator is platform dependent because of course it is...
    let separator = if cfg!(unix) {
        ':'
    } else if cfg!(windows) {
        ';'
    } else {
        ' '
    };

    // TODO: zip files can be interpreted as jars
    let mut class_path = vec!["std/out".into()];

    match (opts.get_args("class_path"), var("CLASSPATH")) {
        // If class path is specified, it overrides CLASSPATH environment variable
        (Some(paths), _) => {
            for path in paths {
                for element in path.split(separator) {
                    class_path.extend(glob(element)
                        .expect("Unable to read file glob")
                        .filter_map(|x| {
                            match x {
                                Ok(x) => Some(x),
                                Err(e) => {
                                    error!("{:?}", e);
                                    None
                                }
                            }
                        }));
                }
            }
        }
        // Use environment variable if possible
        (None, Ok(path)) => {
            for element in path.split(separator) {
                class_path.extend(glob(element)
                    .expect("Unable to read file glob")
                    .filter_map(|x| {
                        match x {
                            Ok(x) => Some(x),
                            Err(e) => {
                                error!("{:?}", e);
                                None
                            }
                        }
                    }));
            }
        },
        // If neither is given, default to user directory
        _ => class_path.push(".".into()),
    };

    let java_dir = var("JAVA_HOME").ok().map(PathBuf::from);

    let class_path = match ClassPath::new(java_dir, Some(class_path)) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error indexing class path: {:?}", e);
            exit(1);
        }
    };

    let mut class_loader = ClassLoader::from_class_path(class_path);
    if let Err(e) = class_loader.preload_class_path() {
        eprintln!("Error loading class path: {:?}", e);
        exit(1);
    }

    let mut jvm = JavaEnv::new(class_loader);

    // class_loader.load_new(&"Simple.class".into()).unwrap();



}
