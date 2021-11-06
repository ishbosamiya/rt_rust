use clap::value_t;
use clap::{App, Arg};
use std::path::PathBuf;

use crate::rasterize::texture;

#[derive(Debug)]
// TODOs: trace_max_depth, environment_transform,
// environment_strength, textures, select_texture_for_shader
pub struct InputArguments {
    run_headless: bool,
    num_threads: Option<usize>,
    width: Option<usize>,
    height: Option<usize>,
    sample_count: Option<usize>,
    envt_map: Option<PathBuf>,
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    trace_max_depth: usize,
    environment_strength: usize,
    texture: Option<PathBuf>,
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
                    .short("t")
                    .help("Number of threads")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("width")
                    .short("w")
                    .help("Width")
                    .requires("height")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("height")
                    .short("h")
                    .help("Height")
                    .requires("width")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("samples")
                    .short("S")
                    .help("Number of Samples")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("environment")
                    .short("E")
                    .help("Environment map path")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("rt_file")
                    .short("r")
                    .help("RT File Path")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("output")
                    .short("o")
                    .help("Output File Path")
                    .requires("headless")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("trace_depth")
                    .short("t")
                    .help("Tracing the Max Depth")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("environment_strength")
                    .short("es")
                    .help("Strength of the environment")
                    .requires("headless")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("texture")
                    .short("tx")
                    .help("Texture Image")
                    .requires("headless")
                    .takes_value(true),
            )
            .get_matches();

        dbg!(InputArguments {
            run_headless: app.is_present("headless"),
            // TODO: default number of threads should be determined by system
            num_threads: value_t!(app, "threads", usize).ok(),
            width: value_t!(app, "width", usize).ok(),
            height: value_t!(app, "height", usize).ok(),
            sample_count: value_t!(app, "samples", usize).ok(),
            envt_map: value_t!(app, "environment", PathBuf).ok().map(|path| {
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
            trace_max_depth: value_t!(app, "trace_depth", usize).unwrap_or(5),
            environment_strength: value_t!(app, "environment_strength", usize).unwrap_or(2),
            texture: value_t!(app, "texture", PathBuf).ok(),
        })
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
        self.envt_map.as_ref()
    }

    pub fn get_rt_file(&self) -> Option<&PathBuf> {
        self.input_path.as_ref()
    }

    pub fn get_output_file(&self) -> Option<&PathBuf> {
        self.output_path.as_ref()
    }
}
