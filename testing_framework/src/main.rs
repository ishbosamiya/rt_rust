extern crate clap;
extern crate serde_json;

use clap::{App, Arg};
use is_executable::IsExecutable;
use serde_json::Value;
use std::fs::File;

use std::path::Path;

// Stores values in JSON
struct Test {
    threads: usize,
    width: i32,
    height: i32,
    trace_depth: usize,
    samples: usize,
    env_map: bool,
    rt_files: String,
}

impl Test {
    fn new(
        threads: usize,
        width: i32,
        height: i32,
        trace_depth: usize,
        samples: usize,
        env_map: bool,
        rt_files: String,
    ) -> Self {
        Self {
            threads,
            width,
            height,
            trace_depth,
            samples,
            env_map,
            rt_files,
        }
    }
}

pub fn read_config(config_path: &Path) -> std::io::Result<()> {
    // Assume configs are stored in one folder only

    let conf = File::open(config_path)?;

    let json: Value = serde_json::from_reader(&conf)?;
    // TODO: Call each test function
    let mut tests: Vec<Test> = Vec::new();
    for value in json.as_object().unwrap().values() {
        let test: Test = Test::new(
            value["threads"].to_string().parse::<usize>().unwrap(),
            value["width"].to_string().parse::<i32>().unwrap(),
            value["height"].to_string().parse::<i32>().unwrap(),
            value["trace_depth"].to_string().parse::<usize>().unwrap(),
            value["samples"].to_string().parse::<usize>().unwrap(),
            value["env_map"].to_string().parse::<bool>().unwrap(),
            value["rt_file_path"].to_string(),
        );
        // Alternatively run each test one by one
        // Does not require tests array
        tests.push(test);
    }

    Ok(())
}

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
