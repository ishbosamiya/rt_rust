use clap::value_t;
use clap::{App, Arg};
use std::path::PathBuf;

#[derive(Debug)]
pub struct InputArguments {
    run_headless: bool,
    num_threads: Option<usize>,
    width: usize,
    height: usize,
    sample_count: usize,
    envt_map: Option<PathBuf>,
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
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
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("height")
                    .short("h")
                    .help("Height")
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
            .get_matches();

        dbg!(InputArguments {
            run_headless: app.is_present("headless"),
            // TODO: default number of threads should be determined by system
            num_threads: value_t!(app, "threads", usize).ok(),
            width: value_t!(app, "width", usize).unwrap_or(200),
            height: value_t!(app, "height", usize).unwrap_or(200),
            sample_count: value_t!(app, "samples", usize).unwrap_or(5),
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
        })
    }

    pub fn get_run_headless(&self) -> bool {
        self.run_headless
    }

    pub fn get_image_width(&self) -> usize {
        self.width
    }

    pub fn get_image_height(&self) -> usize {
        self.height
    }

    pub fn get_samples(&self) -> usize {
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
