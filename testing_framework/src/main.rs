extern crate clap;

use clap::{App, Arg};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub fn is_exec_in_path(path: &Path) -> bool {
    path.exists()
}

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
                .index(1)
                .short("c")
                .long("config")
                .help("Config")
                .value_name("FILE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("exec")
                .required(true)
                .short("e")
                .index(2)
                .help("Executable Location"),
        )
        .arg(
            Arg::with_name("pwd")
                .required(true)
                .short("d")
                .index(3)
                .help("Give Directory"),
        )
        .get_matches();

    // let config = app.value_of("config").unwrap();
    // Checking if path and file are correct
    let file_name = app.value_of("exec").unwrap();
    let dir_name = app.value_of("pwd").unwrap();
    let mut path = PathBuf::from(dir_name);
    path.push(file_name);
    assert!(is_exec_in_path(path.as_path()));
    // Checking for executable file
    let file = File::open(path)?;
    let permissions = file.metadata()?.permissions();
    let is_exec = permissions.mode() & 0o111 != 0;
    assert!(is_exec);

    Ok(())
}
