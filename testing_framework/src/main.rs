extern crate clap;
extern crate serde;
extern crate serde_json;

use clap::{value_t, values_t, App, Arg, ArgMatches};
use ipc_channel::ipc;
use is_executable::IsExecutable;
use nalgebra_glm as glm;
use serde::{Deserialize, Serialize};

use rt::util;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{self, exit};
use std::sync::atomic::{self, AtomicBool};
use std::sync::Arc;
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

/// Overrides for the various parameters applied to all the RT
/// files. It makes it easy to change parameters for the entire
/// configuration.
///
/// Not all parameters of the RT file can be overridden.
#[derive(Debug)]
pub struct RTFileOverrides {
    threads: Option<usize>,
    width: Option<usize>,
    height: Option<usize>,
    trace_max_depth: Option<usize>,
    samples: Option<usize>,
    environment_map: Option<PathBuf>,
    environment_strength: Option<f64>,
    environment_location: Option<glm::DVec3>,
    environment_rotation: Option<glm::DVec3>,
    environment_scale: Option<glm::DVec3>,
}

impl RTFileOverrides {
    pub fn clap_cli<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app.arg(
            Arg::with_name("threads")
                .long("threads")
                .help("Number of threads")
                .takes_value(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("width")
                .long("width")
                .help("Width")
                .takes_value(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("height")
                .long("height")
                .help("Height")
                .takes_value(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("trace-max-depth")
                .long("trace-max-depth")
                .help("Tracing the Max Depth")
                .takes_value(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("samples")
                .long("samples")
                .help("Number of Samples per Pixel")
                .takes_value(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("environment")
                .long("environment")
                .help("Environment map path")
                .takes_value(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("environment-strength")
                .long("environment-strength")
                .help("Strength of the environment")
                .requires("environment")
                .takes_value(true)
                .value_name("strength"),
        )
        .arg(
            Arg::with_name("environment-location")
                .long("environment-location")
                .help("Environment Location")
                .requires("environment")
                .takes_value(true)
                .number_of_values(3)
                .value_names(&["x", "y", "z"]),
        )
        .arg(
            Arg::with_name("environment-rotation")
                .long("environment-rotation")
                .help("Environment Rotation")
                .requires("environment")
                .takes_value(true)
                .number_of_values(3)
                .value_names(&["x", "y", "z"]),
        )
        .arg(
            Arg::with_name("environment-scale")
                .long("environment-scale")
                .help("Environment Scale")
                .requires("environment")
                .takes_value(true)
                .number_of_values(3)
                .value_names(&["x", "y", "z"]),
        )
    }

    pub fn from_clap(app: &ArgMatches<'_>) -> Self {
        Self {
            threads: value_t!(app, "threads", usize).ok(),
            width: value_t!(app, "width", usize).ok(),
            height: value_t!(app, "height", usize).ok(),
            trace_max_depth: value_t!(app, "trace-max-depth", usize).ok(),
            samples: value_t!(app, "samples", usize).ok(),
            environment_map: value_t!(app, "environment", PathBuf).ok().map(|path| {
                if path.is_file() {
                    path
                } else {
                    panic!(
                        "Given environment map path does not point to a file: {}",
                        path.to_str().unwrap()
                    );
                }
            }),
            environment_strength: value_t!(app, "environment-strength", f64).ok(),
            environment_location: values_t!(app, "environment-location", f64)
                .ok()
                .map(|location| glm::vec3(location[0], location[1], location[2])),
            environment_rotation: values_t!(app, "environment-rotation", f64)
                .ok()
                .map(|rotation| glm::vec3(rotation[0], rotation[1], rotation[2])),
            environment_scale: values_t!(app, "environment-scale", f64)
                .ok()
                .map(|scale| glm::vec3(scale[0], scale[1], scale[2])),
        }
    }

    /// Get a reference to the r t file overrides's threads.
    pub fn get_threads(&self) -> Option<&usize> {
        self.threads.as_ref()
    }

    /// Get a reference to the r t file overrides's width.
    pub fn get_width(&self) -> Option<&usize> {
        self.width.as_ref()
    }

    /// Get a reference to the r t file overrides's height.
    pub fn get_height(&self) -> Option<&usize> {
        self.height.as_ref()
    }

    /// Get a reference to the r t file overrides's trace max depth.
    pub fn get_trace_max_depth(&self) -> Option<&usize> {
        self.trace_max_depth.as_ref()
    }

    /// Get a reference to the r t file overrides's samples.
    pub fn get_samples(&self) -> Option<&usize> {
        self.samples.as_ref()
    }

    /// Get a reference to the r t file overrides's environment map.
    pub fn get_environment_map(&self) -> Option<&PathBuf> {
        self.environment_map.as_ref()
    }

    /// Get a reference to the r t file overrides's environment strength.
    pub fn get_environment_strength(&self) -> Option<&f64> {
        self.environment_strength.as_ref()
    }

    /// Get a reference to the r t file overrides's environment location.
    pub fn get_environment_location(&self) -> Option<&glm::DVec3> {
        self.environment_location.as_ref()
    }

    /// Get a reference to the r t file overrides's environment rotation.
    pub fn get_environment_rotation(&self) -> Option<&glm::DVec3> {
        self.environment_rotation.as_ref()
    }

    /// Get a reference to the r t file overrides's environment scale.
    pub fn get_environment_scale(&self) -> Option<&glm::DVec3> {
        self.environment_scale.as_ref()
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

    #[serde(default = "default_textures")]
    textures: Vec<PathBuf>,
    #[serde(default = "default_shader_texture")]
    shader_texture: Vec<(String, usize)>,
    #[serde(default = "default_obj_files")]
    obj_files: Vec<PathBuf>,
    #[serde(default = "default_object_shader")]
    object_shader: Vec<(String, String)>,
}

fn default_textures() -> Vec<PathBuf> {
    vec![]
}

fn default_shader_texture() -> Vec<(String, usize)> {
    vec![]
}

fn default_obj_files() -> Vec<PathBuf> {
    vec![]
}

fn default_object_shader() -> Vec<(String, String)> {
    vec![]
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
            textures: vec![PathBuf::from("example_texture.png")],
            shader_texture: vec![("shader_1".to_string(), 0)],
            obj_files: vec![PathBuf::from("example_obj_file.obj")],
            object_shader: vec![("object_1".to_string(), "shader_1".to_string())],
        }
    }
}

pub fn read_config(config_path: &Path) -> Config {
    let json = std::fs::read_to_string(config_path).unwrap();

    serde_json::from_str(&json).unwrap()
}

/// Wrapper around [`std::process::Command`] so that the complete list
/// of arguments can be stored
pub struct Command {
    command: process::Command,
    complete_string: String,
}

impl Command {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        let complete_string = program.as_ref().to_str().unwrap().to_string();
        Self {
            command: process::Command::new(program),
            complete_string,
        }
    }

    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.command.current_dir(dir);
        self
    }

    pub fn stdout<T: Into<process::Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.command.stdout(cfg);
        self
    }

    pub fn stderr<T: Into<process::Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.command.stderr(cfg);
        self
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.complete_string = format!(
            "{} {}",
            self.complete_string,
            arg.as_ref().to_str().unwrap()
        );
        self.command.arg(arg);
        self
    }

    pub fn spawn(&mut self) -> std::io::Result<process::Child> {
        self.command.spawn()
    }

    /// Get a reference to the command's complete string.
    pub fn get_complete_string(&self) -> &str {
        self.complete_string.as_str()
    }
}

fn main() {
    let sigint_triggered = Arc::new(AtomicBool::new(false));
    {
        let sigint_triggered = sigint_triggered.clone();
        ctrlc::set_handler(move || {
            sigint_triggered.store(true, atomic::Ordering::SeqCst);
            println!("SIGINT or SIGTERM is triggered");
        })
        .expect("Error setting signal handler");
    }

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
        .arg(
            Arg::with_name("dry-run")
                .long("dry-run")
                .help("do not execute any render"),
        );
    let app = RTFileOverrides::clap_cli(app);
    let app = app.get_matches();

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

    let dry_run = app.is_present("dry-run");

    let overrides = RTFileOverrides::from_clap(&app);

    println!("config_path: {}", config_path.to_str().unwrap());
    println!("exec_path: {}", exec_path.to_str().unwrap());
    println!(
        "working_directory_path: {}",
        working_directory_path.to_str().unwrap()
    );
    dbg!(&dry_run);
    dbg!(&overrides);

    // Calling the config
    let config_data = read_config(config_path);

    // Spawning a Process for every iteration of data
    config_data.rt_files.iter().for_each(|file| {
        // setup progress server
        let (progress_server, progress_server_name): (ipc::IpcOneShotServer<u64>, _) =
            ipc::IpcOneShotServer::new().unwrap();

        let mut command = Command::new(std::fs::canonicalize(exec_path).unwrap());
        command
            .current_dir(std::fs::canonicalize(working_directory_path).unwrap())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .arg("--headless")
            .arg("--threads")
            .arg(overrides.get_threads().unwrap_or(&file.threads).to_string())
            .arg("--width")
            .arg(overrides.get_width().unwrap_or(&file.width).to_string())
            .arg("--height")
            .arg(overrides.get_height().unwrap_or(&file.height).to_string())
            .arg("--samples")
            .arg(overrides.get_samples().unwrap_or(&file.samples).to_string())
            .arg("--trace-max-depth")
            .arg(
                overrides
                    .get_trace_max_depth()
                    .unwrap_or(&file.trace_max_depth)
                    .to_string(),
            );
        if let Some(environment) = overrides
            .get_environment_map()
            .or_else(|| file.environment_map.as_ref())
        {
            command.arg("--environment").arg(environment);
        }
        if let Some(environment_strength) = overrides
            .get_environment_strength()
            .or_else(|| file.environment_strength.as_ref())
        {
            command
                .arg("--environment-strength")
                .arg(environment_strength.to_string());
        }
        if let Some(location) = overrides
            .get_environment_location()
            .or_else(|| file.environment_location.as_ref())
        {
            command
                .arg("--environment-location")
                .arg(location[0].to_string())
                .arg(location[1].to_string())
                .arg(location[2].to_string());
        }
        if let Some(rotation) = overrides
            .get_environment_rotation()
            .or_else(|| file.environment_rotation.as_ref())
        {
            command
                .arg("--environment-rotation")
                .arg(rotation[0].to_string())
                .arg(rotation[1].to_string())
                .arg(rotation[2].to_string());
        }
        if let Some(scale) = overrides
            .get_environment_scale()
            .or_else(|| file.environment_scale.as_ref())
        {
            command
                .arg("--environment-scale")
                .arg(scale[0].to_string())
                .arg(scale[1].to_string())
                .arg(scale[2].to_string());
        }
        if !file.textures.is_empty() {
            command.arg("--textures");
            file.textures.iter().for_each(|texture_path| {
                command.arg(texture_path);
            });
        }
        if !file.shader_texture.is_empty() {
            file.shader_texture
                .iter()
                .for_each(|(shader_name, texture_index)| {
                    command.arg("--shader-texture");
                    command.arg(format!("{},{}", shader_name, texture_index));
                });
        }
        if !file.obj_files.is_empty() {
            command.arg("--obj-files");
            file.obj_files.iter().for_each(|obj_file_path| {
                command.arg(obj_file_path);
            });
        }
        if !file.object_shader.is_empty() {
            file.object_shader
                .iter()
                .for_each(|(object_name, shader_name)| {
                    command.arg("--object-shader");
                    command.arg(format!("{},{}", object_name, shader_name));
                });
        }
        command
            .arg("--rt-file")
            .arg(file.rt_path.as_path())
            .arg("--output")
            .arg(file.output_path.as_path());
        command
            .arg("--path-trace-progress-server-name")
            .arg(progress_server_name);

        println!(
            "Tracing RT File: {} with arguments:",
            file.rt_path.to_str().unwrap()
        );
        println!("{}", command.get_complete_string());

        if dry_run {
            return;
        }

        let start_instant = std::time::Instant::now();
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

        // tries to wait for the path trace to finish while updating
        // the progress and returns if the loop should break or not
        let mut path_trace_try_wait = |sigint_triggered: bool| {
            match path_trace_handle.try_wait() {
                Ok(Some(status)) => {
                    // process exits

                    if status.success() {
                        if sigint_triggered {
                            println!(
                                "RT File: {} not fully rendered and generated output: {}",
                                file.rt_path.to_str().unwrap(),
                                file.output_path.to_str().unwrap()
                            );
                        } else {
                            pb.finish();
                            println!(
                                "RT File: {} rendered successfully and generated output: {}",
                                file.rt_path.to_str().unwrap(),
                                file.output_path.to_str().unwrap()
                            );
                        }
                    } else {
                        println!(
                            "RT File: {} failed with exit status: {}",
                            file.rt_path.to_str().unwrap(),
                            status
                        );
                    }

                    println!(
                        "Finished in {}",
                        util::duration_to_string(start_instant.elapsed())
                    );

                    true
                }
                Ok(None) => {
                    if let Ok(progress) = progress_receiver.try_recv() {
                        pb.set(progress);
                    }

                    false
                }
                Err(error) => {
                    panic!(
                        "RT File: {} failed with error: {}",
                        file.rt_path.to_str().unwrap(),
                        error
                    );
                }
            }
        };

        loop {
            let sigint_triggered = sigint_triggered.load(atomic::Ordering::SeqCst);
            if sigint_triggered {
                println!("waiting to finish current file");
                loop {
                    let should_break = path_trace_try_wait(sigint_triggered);
                    if should_break {
                        break;
                    }
                }
                exit(-1);
            }

            let should_break = path_trace_try_wait(sigint_triggered);
            if should_break {
                break;
            }
        }
    });
}
