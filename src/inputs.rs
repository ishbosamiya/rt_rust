use crate::glm;
use clap::{value_t, values_t};
use clap::{App, Arg};
use std::path::PathBuf;

#[derive(Debug)]
// TODOs: trace_max_depth, environment_transform,
// environment_strength, textures, select_texture_for_shader
pub struct InputArguments {
    run_headless: bool,
    num_threads: Option<usize>,
    width: Option<usize>,
    height: Option<usize>,
    sample_count: Option<usize>,
    environment_map: Option<PathBuf>,
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    trace_max_depth: Option<usize>,
    environment_strength: Option<f64>,
    texture: Option<PathBuf>,
    environment_location: Option<glm::DVec3>,
    environment_rotation: Option<glm::DVec3>,
    environment_scale: Option<glm::DVec3>,
    /// If provided with a server name (see crate ipc-channel), a
    /// sender is created that sends a progress update of the path trace.
    path_trace_progress_server_name: Option<String>,
}

// Function to return test args processed using clap via cli
impl InputArguments {
    pub fn read() -> Self {
        let app = App::new("Config-exec")
            .version("1.0")
            .about("Test Command Line Arguements")
            .author("Nobody")
            .arg(
                Arg::with_name("headless")
                    .long("headless")
                    .requires("rt-file")
                    .requires("output"),
            )
            .arg(
                Arg::with_name("threads")
                    .long("threads")
                    .short("t")
                    .help("Number of threads")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("width")
                    .long("width")
                    .short("w")
                    .help("Width")
                    .requires("height")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("height")
                    .long("height")
                    .short("h")
                    .help("Height")
                    .requires("width")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("samples")
                    .long("samples")
                    .short("s")
                    .help("Number of Samples per Pixel")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("environment")
                    .long("environment")
                    .short("e")
                    .help("Environment map path")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("rt-file")
                    .long("rt-file")
                    .short("r")
                    .help("RT File Path")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .help("Output File Path")
                    .requires("headless")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("trace-max-depth")
                    .long("trace-max-depth")
                    .alias("tmd")
                    .help("Tracing the Max Depth")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("environment-strength")
                    .long("environment-strength")
                    .alias("es")
                    .help("Strength of the environment")
                    .requires("environment")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("texture")
                    .long("texture")
                    .alias("tx")
                    .help("Texture Image")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("environment-location")
                    .long("environment-location")
                    .alias("env-loc")
                    .help("Environment Location")
                    .requires("environment")
                    .takes_value(true)
                    .number_of_values(3),
            )
            .arg(
                Arg::with_name("environment-rotation")
                    .long("environment-rotation")
                    .alias("env-rot")
                    .help("Environment Rotation")
                    .requires("environment")
                    .takes_value(true)
                    .number_of_values(3),
            )
            .arg(
                Arg::with_name("environment-scale")
                    .long("environment-scale")
                    .alias("env-scale")
                    .help("Environment Scale")
                    .requires("environment")
                    .takes_value(true)
                    .number_of_values(3),
            )
            .arg(
                Arg::with_name("path-trace-progress-server-name")
                    .long("path-trace-progress-server-name")
                    .help("ipc-channel server name that will receive path trace progress updates")
                    .takes_value(true),
            )
            .get_matches();

        let res = InputArguments {
            run_headless: app.is_present("headless"),
            num_threads: value_t!(app, "threads", usize).ok(),
            width: value_t!(app, "width", usize).ok(),
            height: value_t!(app, "height", usize).ok(),
            sample_count: value_t!(app, "samples", usize).ok(),
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
            input_path: value_t!(app, "rt-file", PathBuf).ok(),
            output_path: value_t!(app, "output", PathBuf).ok(),
            trace_max_depth: value_t!(app, "trace-max-depth", usize).ok(),
            environment_strength: value_t!(app, "environment-strength", f64).ok(),
            texture: value_t!(app, "texture", PathBuf).ok(),
            environment_location: values_t!(app, "environment-location", f64)
                .ok()
                .map(|location| glm::vec3(location[0], location[1], location[2])),
            environment_rotation: values_t!(app, "environment-rotation", f64)
                .ok()
                .map(|rotation| glm::vec3(rotation[0], rotation[1], rotation[2])),
            environment_scale: values_t!(app, "environment-scale", f64)
                .ok()
                .map(|scale| glm::vec3(scale[0], scale[1], scale[2])),
            path_trace_progress_server_name: value_t!(
                app,
                "path-trace-progress-server-name",
                String
            )
            .ok(),
        };

        dbg!(res)
    }

    pub fn get_run_headless(&self) -> bool {
        self.run_headless
    }

    pub fn get_image_width(&self) -> Option<usize> {
        self.width
    }

    pub fn get_image_height(&self) -> Option<usize> {
        self.height
    }

    pub fn get_samples(&self) -> Option<usize> {
        self.sample_count
    }

    pub fn get_num_threads(&self) -> Option<usize> {
        self.num_threads
    }

    pub fn get_environment_map(&self) -> Option<&PathBuf> {
        self.environment_map.as_ref()
    }

    pub fn get_rt_file(&self) -> Option<&PathBuf> {
        self.input_path.as_ref()
    }

    pub fn get_output_file(&self) -> Option<&PathBuf> {
        self.output_path.as_ref()
    }

    pub fn get_trace_max_depth(&self) -> Option<usize> {
        self.trace_max_depth
    }

    pub fn get_environment_strength(&self) -> Option<f64> {
        self.environment_strength
    }

    pub fn get_texture(&self) -> Option<&PathBuf> {
        self.texture.as_ref()
    }

    pub fn get_environment_location(&self) -> Option<&glm::DVec3> {
        self.environment_location.as_ref()
    }

    pub fn get_environment_rotation(&self) -> Option<&glm::DVec3> {
        self.environment_rotation.as_ref()
    }

    pub fn get_environment_scale(&self) -> Option<&glm::DVec3> {
        self.environment_scale.as_ref()
    }

    /// Get a reference to the input arguments's path trace progress server name.
    pub fn get_path_trace_progress_server_name(&self) -> Option<&String> {
        self.path_trace_progress_server_name.as_ref()
    }
}
