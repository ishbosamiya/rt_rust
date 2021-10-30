extern crate clap;

use clap::{App, Arg};
use is_executable::IsExecutable;

use std::path::Path;

// struct ExecTest {
//     name: String,
// }

// impl ExecTest {
//     // fn new() -> Self {
//     //     let app = App::new("test-executable")
//     //         .version("1.0")
//     //         .about("Tests Command Line Arguements")
//     //         .author("Nobody");
//     //     let name = matches.value_of("name").expect("Was Required");
//     // }
// }

fn main() -> std::io::Result<()> {
    //println!("Main");
    let app = App::new("test-exec")
        .version("1.0")
        .about("Tests Command Line Arguements")
        .author("Nobody")
        .arg(
            Arg::with_name("config")
                .required(true)
                .short("c")
                .help("Config File Location")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("exec")
                .required(true)
                .short("e")
                .help("Executable Location")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("working-directory")
                .short("w")
                .help("Working Directory")
                .takes_value(true),
        )
        .get_matches();

    let config_path = Path::new(app.value_of("config").unwrap());
    if !config_path.exists() || !config_path.is_file() {
        eprintln!(
            "config path is invalid or is not a file: {}",
            config_path.to_str().unwrap()
        )
    }

    let exec_path = Path::new(app.value_of("exec").unwrap());
    if !exec_path.exists() || !exec_path.is_executable() {
        eprintln!(
            "executable path is invalid or not an executable: {}",
            exec_path.to_str().unwrap()
        )
    }

    let working_directory_path = if let Some(path) = app.value_of("working-directory") {
        Path::new(path)
    } else {
        exec_path.parent().unwrap()
    };
    if !working_directory_path.exists() || !working_directory_path.is_dir() {
        eprintln!(
            "executable path is invalid or not a directory: {}",
            working_directory_path.to_str().unwrap()
        )
    }

    println!("config_path: {}", config_path.to_str().unwrap());
    println!("exec_path: {}", exec_path.to_str().unwrap());
    println!(
        "working_directory_path: {}",
        working_directory_path.to_str().unwrap()
    );

    Ok(())
}
