extern crate clap;
extern crate serde;
extern crate serde_json;

use clap::{App, Arg};
use is_executable::IsExecutable;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::process::Command;

use std::path::{Path, PathBuf};

// Stores values in JSON

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    rt_files: Vec<RustFileInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RustFileInfo {
    threads: usize,
    width: usize,
    height: usize,
    trace_depth: usize,
    samples: usize,
    env_map: Option<PathBuf>,
    rt_path: PathBuf,
}

impl RustFileInfo {
    fn new(
        threads: usize,
        width: usize,
        height: usize,
        trace_depth: usize,
        samples: usize,
        env_map: Option<PathBuf>,
        rt_path: PathBuf,
    ) -> Self {
        Self {
            threads,
            width,
            height,
            trace_depth,
            samples,
            env_map,
            rt_path,
        }
    }
}

pub fn read_config(config_path: &Path) -> Config {
    // Assume configs are stored in one folder only

    let conf = std::fs::read_to_string(config_path).unwrap();

    let rt_files: Vec<RustFileInfo> = serde_json::from_str(&conf).unwrap();

    Config { rt_files }
}

fn main() -> std::io::Result<()> {
    //println!("Main");
    let app = App::new("Config-exec")
        .version("1.0")
        .about("Configs Command Line Arguements")
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

    // Calling the config
    let config_data = read_config(config_path);

    // Spawning a Process for every iteration of data
    config_data.rt_files.iter().for_each(|f| {
        // TODO: Enter a command using std::command to call executable
    });

    Ok(())
}
