use hashbrown::{HashMap, HashSet};
use std::env;

#[derive(Debug)]
pub enum ArgType {
    Flag,
    Valued,
    Keyed,
}

#[derive(Debug)]
pub struct ArgHandler {
    pub name: &'static str,
    pub aliases: Vec<&'static str>,
    pub arg_type: ArgType,
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
pub struct ManualOpts {
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
        let mut args = get_java_args().into_iter();
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

    pub fn has_flag(&self, key: &'static str) -> bool {
        self.flags.contains(key)
    }

    pub fn get_args(&self, key: &'static str) -> Option<&[String]> {
        self.args.get(key).map(|x|&x[..])
    }
}

pub fn get_java_args() -> Vec<String> {
    let mut args = Vec::new();
    let mut cli_flags = env::args().collect::<Vec<String>>();

    // Push executable name
    args.push(cli_flags.remove(0));

    // Push args from JDK_JAVA_OPTIONS if present
    let env_args = env::var("JDK_JAVA_OPTIONS").unwrap_or_else(|_| String::new());
    let env_args = shell_words::split(&env_args).expect("failed to parse JDK_JAVA_OPTIONS");
    args.extend(env_args);

    // Push remaining executable args
    args.extend(cli_flags);
    args
}

