use clap::{App, Arg};
use jvm::class::constant::Constant;
use jvm::class::{ClassLoader, ClassPath};

fn main() {
    let app = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("class")
                .takes_value(true)
                .required(true)
                .help("Java class to read"),
        )
        .arg(
            Arg::with_name("constants")
                .short("c")
                .long("constants")
                .help("Print the raw constant table as it appears in the class file"),
        )
        .arg(
            Arg::with_name("fields")
                .short("f")
                .long("fields")
                .help("Print the class fields as they appear in the class file"),
        )
        .get_matches();

    let class = app.value_of("class").unwrap();

    let class_path = ClassPath::new(None, Some(vec![".".into()])).unwrap();
    let mut class_loader = ClassLoader::from_class_path(class_path);
    class_loader.preload_class_path().unwrap();

    // println!("Reading: {}", class);

    if !class_loader.attempt_load(class).unwrap() {
        panic!("Unable to load class: {:?}", class)
    }

    let raw_class = class_loader.class(class).unwrap();
    println!("Reading: {} extends {}", class, raw_class.super_class());

    if app.is_present("constants") {
        println!("Constant Table:");
        let mut idx = 1;
        for constant in &raw_class.constants {
            match constant {
                Constant::Placeholder => {}
                x => println!("\t{}/{}: {:?}", idx, raw_class.constants.len(), x),
            };
            idx += 1;
        }
    }

    if app.is_present("fields") {
        for field in &raw_class.fields {
            println!(
                "{} ({}): {:?}",
                field.name(&raw_class.constants).unwrap(),
                field.descriptor(&raw_class.constants).unwrap(),
                field
            );
        }
    }
}
