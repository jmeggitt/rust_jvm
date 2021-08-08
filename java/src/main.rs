// use clap::{Arg, App, Subcommand, ArgGroup};
use pretty_env_logger::formatted_builder;
use pretty_env_logger::env_logger::Target;
use log::LevelFilter;
// use clap::Clap;
use std::collections::{HashMap, HashSet};

use std::env;

// #[derive(Clap, Debug)]
// #[clap(version = env ! ("CARGO_PKG_VERSION"), author = env ! ("CARGO_PKG_AUTHORS"))]
// struct Opts {
//     // #[clap(short, long)]
//     // verbose: bool,
//     // #[clap(short, long)]
//     // class_path: Vec<String>,
//     // #[clap(long)]
//     // jar: bool,
//     // // #[clap(required)]
//     // target: String,
//     // // #[clap(multiple)]
//     // args: Vec<String>,
//     #[clap(short, long, number_of_values = 1)]
//     foo: Vec<String>,
//     #[clap(allow_hyphen_values = true)]
//     inputs: Vec<String>,
// }

#[derive(Debug)]
enum ArgType {
    Flag,
    Valued,
    Keyed,
}

#[derive(Debug)]
struct ArgHandler {
    name: &'static str,
    aliases: Vec<&'static str>,
    arg_type: ArgType,
}

impl ArgHandler {
    pub fn named(name: &'static str, arg_type: ArgType) -> Self {
        ArgHandler {
            name,
            aliases: Vec::new(),
            arg_type,
        }
    }

    pub fn aliases(mut self, aliases: &'static [&'static str]) -> Self {
        self.aliases.extend_from_slice(aliases);
        self
    }
}

#[derive(Debug, Default)]
struct ManualOpts {
    schemas: Vec<ArgHandler>,
    flags: HashSet<&'static str>,
    args: HashMap<&'static str, Vec<String>>,
    key_args: HashMap<&'static str, HashMap<String, Option<String>>>,
    program_args: Vec<String>,
}

impl ManualOpts {
    pub fn arg(mut self, arg: ArgHandler) -> Self {
        self.schemas.push(arg);
        self
    }

    pub fn parse(mut self) -> Self {
        let mut args = Vec::new();

        let mut cli_flags = env::args().collect::<Vec<String>>();
        args.push(cli_flags.remove(0));

        let env_args = env::var("JDK_JAVA_OPTIONS").unwrap_or_else(|_| String::new());
        let env_args = shell_words::split(&env_args).expect("failed to parse JDK_JAVA_OPTIONS");
        args.extend(env_args);

        args.extend(cli_flags);
        let mut args = args.into_iter();
        let _executable = args.next();

        'parser: while let Some(arg) = args.next() {
            for schema in &self.schemas {
                for alias in &schema.aliases {
                    match schema.arg_type {
                        ArgType::Flag => {
                            if arg == *alias {
                                self.flags.insert(schema.name);
                                continue 'parser;
                            }
                        }
                        ArgType::Valued => {
                            if arg == *alias {
                                let value = args.next().expect("Expected argument value");
                                self.args.entry(schema.name).or_insert_with(|| Vec::new()).push(value);
                                continue 'parser;
                            }
                        }
                        ArgType::Keyed => {
                            if arg.starts_with(alias) {
                                if arg.len() == alias.len() {
                                    panic!("Expected key after {}", alias);
                                }

                                let (key, value) = match arg.find("=") {
                                    Some(v) => (arg[alias.len()..v].to_string(), Some(arg[v + 1..].to_string())),
                                    None => (arg[alias.len()..].to_string(), None),
                                };

                                let mut keyed_arg = self.key_args.entry(schema.name).or_insert_with(|| HashMap::new());
                                keyed_arg.insert(key, value);

                                continue 'parser;
                            }
                        }
                    };
                }
            }

            self.program_args.push(arg);
            self.program_args.extend(&mut args);
        }

        self
    }
}

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

    // let opts = Opts::parse();
    // let app = App::new("Rusty Java")
    //     .version(env!("CARGO_PKG_VERSION"))
    //     .author(env!("CARGO_PKG_AUTHORS"))
    //     .about(env!("CARGO_PKG_DESCRIPTION"))
    //     .arg(Arg::with_name("v")
    //         .short("v")
    //         .multiple(true)
    //         .help("Enables verbose output"))
    //     .arg(Arg::with_name("classpath")
    //         .long("class-path")
    //         .long("cp")
    //         .takes_value(true)
    //         .multiple(true)
    //         .help("Adds a file or directory to the class search path"))
    //     .arg(Arg::with_name("jar")
    //         .long("jar")
    //         .help("Launches from a given jar file"))
    //     .arg(Arg::with_name("target")
    //         .takes_value(true)
    //         .required(true)
    //         .help("Launches"))
    //     .arg(Arg::with_name("args")
    //         .takes_value(true)
    //         .multiple(true)
    //         .help("Values to pass to the java program"));

    // let app = App::new("program")
    //     .arg(Arg::with_name("configfile")
    //         .takes_value(true))
    //     .arg(Arg::with_name("zip")
    //         .long("from-zip")
    //         .value_name("zipfile")
    //         .takes_value(true))
    //     .arg(Arg::with_name("args")
    //         .takes_value(true)
    //         .multiple(true));

    // TODO: How do I support JDK_JAVA_OPTIONS?

    // let matches = app.get_matches();

    println!("{:?}", &opts);

    // let log_level = match matches.is_present("verbose") {
    //     true => LevelFilter::Debug,
    //     false => LevelFilter::Error,
    // };
    //
    // formatted_builder()
    //     .target(Target::Stdout)
    //     .filter_level(log_level)
    //     .init();
}
