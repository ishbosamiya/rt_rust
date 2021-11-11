extern crate clap;
extern crate serde;
extern crate serde_json;

use clap::{value_t, App, Arg};
use ipc_channel::ipc;
use is_executable::IsExecutable;
use nalgebra_glm as glm;
use serde::{Deserialize, Serialize};
use std::process::{exit, Command};

use std::path::{Path, PathBuf};
use std::time::Duration;

// Stores values in JSON

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    rt_files: Vec<RustFileInfo>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rt_files: vec![RustFileInfo::default()],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RustFileInfo {
    rt_path: PathBuf,
    output_path: PathBuf,
    threads: usize,
    width: usize,
    height: usize,
    trace_max_depth: usize,
    samples: usize,
    environment_map: Option<PathBuf>,
    environment_strength: Option<f64>,
    environment_location: Option<glm::DVec3>,
    environment_rotation: Option<glm::DVec3>,
    environment_scale: Option<glm::DVec3>,
}

impl Default for RustFileInfo {
    fn default() -> Self {
        Self {
            rt_path: PathBuf::from("example.rt"),
            output_path: PathBuf::from("output.image"),
            threads: 0,
            width: 200,
            height: 200,
            trace_max_depth: 10,
            samples: 20,
            environment_map: Some(PathBuf::from("example.hdr")),
            environment_strength: Some(1.0),
            environment_location: Some(glm::vec3(0.0, 0.0, 0.0)),
            environment_rotation: Some(glm::vec3(0.0, 0.0, 0.0)),
            environment_scale: Some(glm::vec3(1.0, 1.0, 1.0)),
        }
    }
}

pub fn read_config(config_path: &Path) -> Config {
    let json = std::fs::read_to_string(config_path).unwrap();

    serde_json::from_str(&json).unwrap()
}

fn main() {
    let app = App::new("Config-exec")
        .version("1.0")
        .about("Configs Command Line Arguments")
        .author("Nobody")
        .arg(
            Arg::with_name("generate-default-config")
                .long("generate-default-config")
                .takes_value(true)
                .conflicts_with_all(&["config", "exec", "working-directory"])
                .help("Generation default config at the given file path"),
        )
        .arg(
            Arg::with_name("config")
                .long("config")
                .required_unless("generate-default-config")
                .short("c")
                .help("Config File Location")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("exec")
                .long("exec")
                .required_unless("generate-default-config")
                .short("e")
                .help("Executable Location")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("working-directory")
                .long("working-directory")
                .short("w")
                .help("Working Directory")
                .takes_value(true),
        )
        .get_matches();

    if let Some(path) = clap::value_t!(app, "generate-default-config", PathBuf).ok() {
        let json = serde_json::to_string_pretty(&Config::default()).unwrap();
        std::fs::write(path, json).unwrap();
        exit(0);
    }

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
    config_data.rt_files.iter().for_each(|file| {
        // setup progress server
        let (progress_server, progress_server_name): (ipc::IpcOneShotServer<u64>, _) =
            ipc::IpcOneShotServer::new().unwrap();

        let mut command = Command::new(exec_path);
        command
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .arg("--headless")
            .arg("--threads")
            .arg(file.threads.to_string())
            .arg("--width")
            .arg(file.width.to_string())
            .arg("--height")
            .arg(file.height.to_string())
            .arg("--samples")
            .arg(file.samples.to_string())
            .arg("--trace-max-depth")
            .arg(file.trace_max_depth.to_string());
        if let Some(path) = file.environment_map.as_ref() {
            command.arg("--environment").arg(path);
        }
        if let Some(strength) = file.environment_strength {
            command
                .arg("--environment-strength")
                .arg(strength.to_string());
        }
        if let Some(location) = file.environment_location.as_ref() {
            command
                .arg("--environment-location")
                .arg(location[0].to_string())
                .arg(location[1].to_string())
                .arg(location[2].to_string());
        }
        if let Some(rotation) = file.environment_rotation.as_ref() {
            command
                .arg("--environment-rotation")
                .arg(rotation[0].to_string())
                .arg(rotation[1].to_string())
                .arg(rotation[2].to_string());
        }
        if let Some(scale) = file.environment_scale.as_ref() {
            command
                .arg("--environment-scale")
                .arg(scale[0].to_string())
                .arg(scale[1].to_string())
                .arg(scale[2].to_string());
        }
        command
            .arg("--rt-file")
            .arg(file.rt_path.as_path())
            .arg("--output")
            .arg(file.output_path.as_path());
        command
            .arg("--path-trace-progress-server-name")
            .arg(progress_server_name);

        let mut path_trace_handle = command.spawn().expect("Error in spawing");

        // Fixes a bug where the child has some error before sending
        // the first packet to the server thus it has already exited
        // but progress_server is gonna wait to receive a packet
        // leading to a complete block of the testing
        // framework. Simplest fix is to assume that the error will
        // happen in under 500ms so just wait for that long and check
        // if child has exited yet.
        //
        // TODO: find a better way to fix this sort of bug
        std::thread::sleep(Duration::from_millis(500));
        if let Ok(Some(status)) = path_trace_handle.try_wait() {
            println!(
                "RT File: {} failed with exit status: {}",
                file.rt_path.to_str().unwrap(),
                status
            );
            return;
        }

        // accept the connect, must be done after spawning the child
        // process since it will wait for the first message to be
        // passed to the server
        let (progress_receiver, total_number_of_samples) = progress_server.accept().unwrap();

        let mut pb = pbr::ProgressBar::new(total_number_of_samples);

        pb.message(&format!("Tracing {}: ", file.rt_path.to_str().unwrap()));

        loop {
            match path_trace_handle.try_wait() {
                Ok(Some(status)) => {
                    // process exits

                    pb.finish();

                    if status.success() {
                        println!(
                            "RT File: {} rendered successfully and generated output: {}",
                            file.rt_path.to_str().unwrap(),
                            file.output_path.to_str().unwrap()
                        );
                    } else {
                        println!(
                            "RT File: {} failed with exit status: {}",
                            file.rt_path.to_str().unwrap(),
                            status
                        );
                    }

                    break;
                }
                Ok(None) => {
                    if let Ok(progress) = progress_receiver.try_recv() {
                        pb.set(progress);
                    } else {
                        pb.tick();
                    }
                }
                Err(error) => {
                    panic!(
                        "RT File: {} failed with error: {}",
                        file.rt_path.to_str().unwrap(),
                        error
                    );
                }
            }
        }
    });
}
