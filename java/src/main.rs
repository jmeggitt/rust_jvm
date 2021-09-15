use glob::glob;
use log::LevelFilter;
use log::{error, info};
use std::env::var;

mod args;

use args::*;

// use jvm::class::{ClassLoader, ClassPath};
// use jvm::jvm::JavaEnv;
use jni::sys::{JNI_CreateJavaVM, JNI_GetDefaultJavaVMInitArgs, JavaVMInitArgs, JNI_ERR, JNI_TRUE, JNI_VERSION_1_8, JNIEnv, JNINativeInterface_, jvalue};
use std::ffi::{c_void, CString};
use std::path::PathBuf;
use std::process::exit;
use std::ptr::{null_mut, null};

#[link(name = "jvm")]
extern "C" {}

fn main() {
    let opts = ManualOpts::default()
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
    let _log_level = match opts.has_flag("verbose") {
        true => LevelFilter::Debug,
        false => LevelFilter::Error,
    };

    // formatted_builder()
    //     .target(Target::Stdout)
    //     .filter_level(log_level)
    //     .init();

    if opts.has_flag("verbose") {
        info!("Arguments: {:?}", get_java_args());
        info!("Running in verbose mode");
    }

    if opts.program_args.is_empty() {
        eprintln!("You must specify a main class or jar file to run!");
        exit(1);
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
    let mut class_path = vec!["java_std/out".into()];

    match (opts.get_args("class_path"), var("CLASSPATH")) {
        // If class path is specified, it overrides CLASSPATH environment variable
        (Some(paths), _) => {
            for path in paths {
                for element in path.split(separator) {
                    class_path.extend(glob(element).expect("Unable to read file glob").filter_map(
                        |x| match x {
                            Ok(x) => Some(x),
                            Err(e) => {
                                error!("{:?}", e);
                                None
                            }
                        },
                    ));
                }
            }
        }
        // Use environment variable if possible
        (None, Ok(path)) => {
            for element in path.split(separator) {
                class_path.extend(glob(element).expect("Unable to read file glob").filter_map(
                    |x| match x {
                        Ok(x) => Some(x),
                        Err(e) => {
                            error!("{:?}", e);
                            None
                        }
                    },
                ));
            }
        }
        // If neither is given, default to user directory
        _ => class_path.push(".".into()),
    };

    class_path.push("/mnt/c/Users/Jasper/CLionProjects/JavaClassTests/java_std/out".into());

    // If running a jar, add it to the class path
    if opts.has_flag("jar") {
        class_path.push(PathBuf::from(&opts.program_args[0]));
    }

    // set_var("LD_LIBRARY_PATH", "/mnt/c/Users/Jasper/CLionProjects/JavaClassTests/target/release/libjvm.so");
    // set_var("LD_LIBRARY_PATH", "/mnt/c/Users/Jasper/CLionProjects/JavaClassTests/target/release/deps:/mnt/c/Users/Jasper/CLionProjects/JavaClassTests/target/release:/home/jaspermeggitt/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib:/home/jaspermeggitt/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib");
    // set_var("_", "/home/jaspermeggitt/.cargo/bin/cargo");

    unsafe {
        // let lib = Library::new(
        //     "/mnt/c/Users/Jasper/CLionProjects/JavaClassTests/target/release/libjvm.so",
        // )
        // .unwrap();
        //
        // let get_jvm_init_args: Symbol<unsafe extern "system" fn(args: *mut c_void) -> jint> =
        //     lib.get(b"JNI_GetDefaultJavaVMInitArgs").unwrap();
        // let create_jvm: Symbol<
        //     unsafe extern "system" fn(
        //         pvm: *mut *mut JavaVM,
        //         penv: *mut *mut JNIEnv,
        //         args: *mut c_void,
        //     ) -> jint,
        // > = lib.get(b"JNI_CreateJavaVM").unwrap();

        let mut args = JavaVMInitArgs {
            version: JNI_VERSION_1_8,
            nOptions: 0,
            options: null_mut(),
            ignoreUnrecognized: JNI_TRUE,
        };

        if JNI_GetDefaultJavaVMInitArgs(&mut args as *mut _ as *mut c_void) == JNI_ERR {
            panic!("Unable to get jni init args");
        }

        let mut jvm = null_mut();
        let mut env = null_mut();

        if JNI_CreateJavaVM(
            &mut jvm,
            &mut env as *mut _ as _,
            &args as *const JavaVMInitArgs as _,
        ) == JNI_ERR
        {
            panic!("Unable to create Java VM");
        }

        let env: *mut JNIEnv = env;
        let interface: &JNINativeInterface_ = &**env;

        let class = CString::new("Simple").unwrap();
        let target = interface.FindClass.unwrap()(env, class.as_ptr());
        let method = interface.GetStaticMethodID.unwrap()(env, target, "main\0".as_ptr() as _, "([Ljava/lang/String;)V\0".as_ptr() as _);

        let args = [jvalue {l: null_mut()}; 2];
        interface.CallStaticVoidMethodA.unwrap()(env, target, method, &args[0] as *const _)
        // env.CallStaticVoidMethodA
    }

    // let java_dir = var("JAVA_HOME").ok().map(PathBuf::from);
    //
    // let class_path = match ClassPath::new(java_dir, Some(class_path)) {
    //     Ok(v) => v,
    //     Err(e) => {
    //         eprintln!("Error indexing class path: {:?}", e);
    //         exit(1);
    //     }
    // };
    //
    // let mut class_loader = ClassLoader::from_class_path(class_path);
    // if let Err(e) = class_loader.preload_class_path() {
    //     eprintln!("Error loading class path: {:?}", e);
    //     exit(1);
    // }
    //
    // // Find the main class from the jar
    // let main_class = if opts.has_flag("jar") {
    //     let target_jar = PathBuf::from(&opts.program_args[0]);
    //     match class_loader
    //         .loaded_jars
    //         .get(&target_jar)
    //         .unwrap()
    //         .manifest
    //         .main_class()
    //     {
    //         Some(v) => v,
    //         None => {
    //             eprintln!("{} does not have a main class!", target_jar.display());
    //             exit(1);
    //         }
    //     }
    // } else {
    //     opts.program_args[0].replace('.', "/")
    // };
    //
    // let mut jvm = JavaEnv::new(class_loader);
    // if let Err(e) = jvm.entry_point(&main_class, opts.program_args) {
    //     eprintln!("An error occurred while attempting to run main:\n{}", e);
    // }
}
