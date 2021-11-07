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
                    .requires("rt_file")
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
                    .short("S")
                    .help("Number of Samples")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("environment")
                    .long("environment")
                    .short("E")
                    .help("Environment map path")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("rt_file")
                    .long("rt_file")
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
                Arg::with_name("trace_max_depth")
                    .long("trace_max_depth")
                    .alias("tmd")
                    .help("Tracing the Max Depth")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("environment_strength")
                    .long("environment_strength")
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
                Arg::with_name("environment_location")
                    .long("environment_location")
                    // .alias("envt-loc")
                    .help("Environment Location")
                    .requires("environment")
                    .takes_value(true)
                    .number_of_values(3),
            )
            .arg(
                Arg::with_name("environment_rotation")
                    .long("environment_rotation")
                    // .alias("envt-rot")
                    .help("Environment Rotation")
                    .requires("environment")
                    .takes_value(true)
                    .number_of_values(3),
            )
            .arg(
                Arg::with_name("environment_scale")
                    .long("environment_scale")
                    .alias("envt-scale")
                    .help("Environment Scale")
                    .requires("environment")
                    .takes_value(true)
                    .number_of_values(3),
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
            input_path: value_t!(app, "rt_file", PathBuf).ok(),
            output_path: value_t!(app, "output", PathBuf).ok(),
            trace_max_depth: value_t!(app, "trace_max_depth", usize).ok(),
            environment_strength: value_t!(app, "environment_strength", f64).ok(),
            texture: value_t!(app, "texture", PathBuf).ok(),
            environment_location: values_t!(app, "environment_location", f64)
                .ok()
                .map(|location| glm::vec3(location[0], location[1], location[2])),
            environment_rotation: values_t!(app, "environment_rotation", f64)
                .ok()
                .map(|rotation| glm::vec3(rotation[0], rotation[1], rotation[2])),
            environment_scale: values_t!(app, "environment_scale", f64)
                .ok()
                .map(|scale| glm::vec3(scale[0], scale[1], scale[2])),
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
}
