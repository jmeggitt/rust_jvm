use glob::glob;
use log::LevelFilter;
use log::{error, info};
use std::env::var;

mod args;

use args::*;

use jni::sys::{
    jvalue, JNIEnv, JNINativeInterface_, JNI_CreateJavaVM, JNI_GetDefaultJavaVMInitArgs,
    JavaVMInitArgs, JNI_ERR, JNI_TRUE, JNI_VERSION_1_8,
};
use std::ffi::{c_void, CString};
use std::path::PathBuf;
use std::process::exit;
use std::ptr::null_mut;

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

    unsafe {
        // #[cfg(windows)]
        // libloading::Library::new("target/release/jvm.dll").unwrap();

        // TODO: Pass arguments to jvm
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
        let method = interface.GetStaticMethodID.unwrap()(
            env,
            target,
            "main\0".as_ptr() as _,
            "([Ljava/lang/String;)V\0".as_ptr() as _,
        );

        let args = [jvalue { l: null_mut() }; 2];
        interface.CallStaticVoidMethodA.unwrap()(env, target, method, &args[0] as *const _)
    }
}
