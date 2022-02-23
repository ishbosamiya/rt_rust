use clap::{value_t, values_t};
use clap::{App, Arg};
use itertools::Itertools;
use quick_renderer::{
    camera::{self, Camera},
    texture::TextureRGBAFloat,
};

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::{
    file, glm,
    image::Image,
    path_trace::{
        self, bsdfs::utils::ColorPicker, environment::Environment, shader_list::ShaderList,
        texture_list::TextureList, RayTraceParams,
    },
    scene::Scene,
    transform::Transform,
};

#[derive(Debug)]
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
    textures: Vec<PathBuf>,
    environment_location: Option<glm::DVec3>,
    environment_rotation: Option<glm::DVec3>,
    environment_scale: Option<glm::DVec3>,
    /// If provided with a server name (see crate ipc-channel), a
    /// sender is created that sends a progress update of the path trace.
    path_trace_progress_server_name: Option<String>,
    shader_texture: Vec<(String, usize)>,
    mesh_files: Vec<PathBuf>,
    /// A list of object and shader pairs, assigns a shader with the
    /// given shader name to the object with given object name.
    object_shader: Vec<(String, String)>,
}

// Function to return test args processed using clap via cli
impl InputArguments {
    fn get_app() -> App<'static, 'static> {
        App::new("Config-exec")
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
                    .takes_value(true)
                    .value_name("depth"),
            )
            .arg(
                Arg::with_name("environment-strength")
                    .long("environment-strength")
                    .alias("es")
                    .help("Strength of the environment")
                    .requires("environment")
                    .takes_value(true)
                    .value_name("strength"),
            )
            .arg(
                Arg::with_name("textures")
                    .long("textures")
                    .alias("tx")
                    .help("Path to one or more textures")
                    .takes_value(true)
                    .multiple(true),
            )
            .arg(
                Arg::with_name("environment-location")
                    .long("environment-location")
                    .alias("env-loc")
                    .help("Environment Location")
                    .requires("environment")
                    .takes_value(true)
                    .number_of_values(3)
                    .value_names(&["x", "y", "z"]),
            )
            .arg(
                Arg::with_name("environment-rotation")
                    .long("environment-rotation")
                    .alias("env-rot")
                    .help("Environment Rotation")
                    .requires("environment")
                    .takes_value(true)
                    .number_of_values(3)
                    .value_names(&["x", "y", "z"]),
            )
            .arg(
                Arg::with_name("environment-scale")
                    .long("environment-scale")
                    .alias("env-scale")
                    .help("Environment Scale")
                    .requires("environment")
                    .takes_value(true)
                    .number_of_values(3)
                    .value_names(&["x", "y", "z"]),
            )
            .arg(
                Arg::with_name("path-trace-progress-server-name")
                    .long("path-trace-progress-server-name")
                    .help("ipc-channel server name that will receive path trace progress updates")
                    .takes_value(true)
                    .value_name("server-name"),
            )
            .arg(
                Arg::with_name("shader-texture")
                    .long("shader-texture")
                    .help("Assign texture to shader given the shader name and texture index")
                    .takes_value(true)
                    .number_of_values(2)
                    .value_names(&["shader-name", "texture-index"])
                    .multiple(true)
                    .use_delimiter(true)
                    .require_delimiter(true),
            )
            .arg(
                Arg::with_name("mesh-files")
                    .long("mesh-files")
                    .alias("obj-files")
                    .help("Load specified mesh files into the scene prior to render")
                    .takes_value(true)
                    .multiple(true),
            )
            .arg(
                Arg::with_name("object-shader")
                    .long("object-shader")
                    .help("Assign shader to object given the object name and shader name")
                    .takes_value(true)
                    .number_of_values(2)
                    .value_names(&["object-name", "shader-name"])
                    .multiple(true)
                    .use_delimiter(true)
                    .require_delimiter(true),
            )
    }

    fn from_matches(matches: clap::ArgMatches) -> Self {
        InputArguments {
            run_headless: matches.is_present("headless"),
            num_threads: value_t!(matches, "threads", usize).ok(),
            width: value_t!(matches, "width", usize).ok(),
            height: value_t!(matches, "height", usize).ok(),
            sample_count: value_t!(matches, "samples", usize).ok(),
            environment_map: value_t!(matches, "environment", PathBuf).ok().map(|path| {
                if path.is_file() {
                    path
                } else {
                    panic!(
                        "Given environment map path does not point to a file: {}",
                        path.to_str().unwrap()
                    );
                }
            }),
            input_path: value_t!(matches, "rt-file", PathBuf).ok(),
            output_path: value_t!(matches, "output", PathBuf).ok(),
            trace_max_depth: value_t!(matches, "trace-max-depth", usize).ok(),
            environment_strength: value_t!(matches, "environment-strength", f64).ok(),
            textures: values_t!(matches, "textures", PathBuf).map_or(vec![], |textures| textures),
            environment_location: values_t!(matches, "environment-location", f64)
                .ok()
                .map(|location| glm::vec3(location[0], location[1], location[2])),
            environment_rotation: values_t!(matches, "environment-rotation", f64)
                .ok()
                .map(|rotation| glm::vec3(rotation[0], rotation[1], rotation[2])),
            environment_scale: values_t!(matches, "environment-scale", f64)
                .ok()
                .map(|scale| glm::vec3(scale[0], scale[1], scale[2])),
            path_trace_progress_server_name: value_t!(
                matches,
                "path-trace-progress-server-name",
                String
            )
            .ok(),
            shader_texture: matches
                .values_of("shader-texture")
                .map_or(vec![], |shader_texture| {
                    shader_texture
                        .tuples()
                        .map(|(shader, texture)| (shader.to_string(), texture.parse().unwrap()))
                        .collect()
                }),
            mesh_files: values_t!(matches, "mesh-files", PathBuf)
                .map_or(vec![], |mesh_files| mesh_files),
            object_shader: matches
                .values_of("object-shader")
                .map_or(vec![], |object_shader| {
                    object_shader
                        .tuples()
                        .map(|(object, shader)| (object.to_string(), shader.to_string()))
                        .collect()
                }),
        }
    }

    /// read the input arguments from the command line arguments
    pub fn read_cli() -> Self {
        dbg!(Self::from_matches(Self::get_app().get_matches()))
    }

    pub fn read_string(args: String) -> Self {
        dbg!(Self::from_matches(
            Self::get_app()
                .setting(clap::AppSettings::NoBinaryName)
                .get_matches_from(args.split(' ')),
        ))
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

    pub fn get_textures(&self) -> &[PathBuf] {
        self.textures.as_slice()
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

    /// Get a reference to the input arguments's shader texture.
    pub fn get_shader_texture(&self) -> &[(String, usize)] {
        self.shader_texture.as_slice()
    }

    /// Get a reference to the input arguments's obj files.
    pub fn get_obj_files(&self) -> &[PathBuf] {
        self.mesh_files.as_slice()
    }

    /// Get a reference to the input arguments's object shader.
    pub fn get_object_shader(&self) -> &[(String, String)] {
        self.object_shader.as_slice()
    }

    /// generates most of the necessary render info from the input
    /// arguments
    ///
    /// returns the camera, scene, shader list, texture list and the
    /// environment required for rendering
    #[allow(clippy::type_complexity)]
    pub fn generate_render_info(
        &self,
    ) -> (
        RayTraceParams,
        Arc<RwLock<Scene>>,
        Arc<RwLock<ShaderList>>,
        Arc<RwLock<TextureList>>,
        Arc<RwLock<Environment>>,
    ) {
        let image_width = self
            .get_image_width()
            .unwrap_or_else(crate::default_image_width);
        let image_height = self
            .get_image_height()
            .unwrap_or_else(crate::default_image_height);

        let path_trace_camera = Arc::new(RwLock::new({
            let camera_focal_length = 50.0;
            let camera_sensor_width = 36.0;
            let camera_position = glm::vec3(0.0, 0.0, 3.0);
            let aspect_ratio = image_width as f64 / image_height as f64;
            let camera_sensor_height = camera_sensor_width / aspect_ratio;
            let mut camera = Camera::new(
                camera_position,
                glm::vec3(0.0, 1.0, 0.0),
                270.0,
                0.0,
                45.0,
                Some(camera::Sensor::new(
                    camera_sensor_width,
                    camera_sensor_height,
                )),
            );
            camera.set_focal_length(camera_focal_length);
            camera
        }));

        let scene = Arc::new(RwLock::new({
            let mut scene = Scene::new();
            scene.build_bvh(0.01);
            scene
        }));

        let shader_list = Arc::new(RwLock::new({
            let mut shader_list = ShaderList::new();

            shader_list.add_shader(Box::new(path_trace::shaders::Lambert::new(
                path_trace::bsdfs::lambert::Lambert::default(),
            )));
            shader_list.add_shader(Box::new(path_trace::shaders::Lambert::new(
                path_trace::bsdfs::lambert::Lambert::new(glm::vec3(1.0, 0.0, 0.0)),
            )));
            shader_list.add_shader(Box::new(path_trace::shaders::Glossy::new(
                path_trace::bsdfs::glossy::Glossy::default(),
            )));
            shader_list.add_shader(Box::new(path_trace::shaders::Emissive::new(
                path_trace::bsdfs::emissive::Emissive::new(glm::vec3(1.0, 0.4, 1.0), 5.0),
            )));

            shader_list
        }));

        let texture_list = Arc::new(RwLock::new(TextureList::new()));

        let environment = Arc::new(RwLock::new(Environment::default()));

        if let Some(path) = self.get_rt_file() {
            file::load_rt_file(
                &path,
                scene.clone(),
                shader_list.clone(),
                path_trace_camera.clone(),
                environment.clone(),
            );
        }

        // set environment map from the given path overriding the
        // environment map stored in the rt file
        if let Some(path) = self.get_environment_map() {
            let hdr = image::codecs::hdr::HdrDecoder::new(std::io::BufReader::new(
                std::fs::File::open(path).unwrap(),
            ))
            .unwrap();
            let width = hdr.metadata().width as _;
            let height = hdr.metadata().height as _;
            let image = Image::from_vec_rgb_f32(&hdr.read_image_hdr().unwrap(), width, height);
            *environment.write().unwrap() = Environment::new(
                image,
                self.get_environment_strength()
                    .unwrap_or_else(crate::default_environment_strength),
                Transform::new(
                    self.get_environment_location()
                        .map_or(glm::zero(), |location| *location),
                    self.get_environment_rotation()
                        .map_or(glm::zero(), |rotation| *rotation),
                    self.get_environment_scale()
                        .map_or(glm::vec3(1.0, 1.0, 1.0), |scale| *scale),
                ),
            );
        }

        // if image width or image height are present in the arguments,
        // must overide image width and height and also the camera
        if let Some((image_width, image_height)) =
            self.get_image_width().zip(self.get_image_height())
        {
            path_trace_camera
                .write()
                .unwrap()
                .get_sensor_mut()
                .as_mut()
                .unwrap()
                .change_aspect_ratio(image_width as f64 / image_height as f64);
        }

        // add more textures to texture_list if provided in the arguments
        self.get_textures().iter().for_each(|path| {
            texture_list.write().unwrap().add_texture(
                TextureRGBAFloat::load_from_disk(path).unwrap_or_else(|| {
                    panic!(
                        "could not load the texture from specified path: {}",
                        path.to_str().unwrap()
                    )
                }),
            );
        });

        // assign texture to shader
        self.get_shader_texture()
            .iter()
            .for_each(|(shader_name, texture_index)| {
                let texture_id = texture_list.read().unwrap().get_texture_ids()[*texture_index];
                shader_list
                    .write()
                    .unwrap()
                    .get_shaders_mut()
                    .find(|shader| shader.get_shader_name() == shader_name)
                    .unwrap_or_else(|| panic!("no shader found with shader name: {}", shader_name))
                    .get_bsdf_mut()
                    .set_base_color(ColorPicker::Texture(Some(texture_id)));
            });

        // add more objects to the scene (loading obj files)
        {
            self.get_obj_files().iter().for_each(|obj_file_path| {
                crate::load_meshes(obj_file_path)
                    .drain(0..)
                    .for_each(|object| {
                        scene.write().unwrap().add_object(Box::new(object));
                    });
            });

            // update scene bvh
            {
                let mut scene = scene.write().unwrap();
                scene.apply_model_matrices();

                scene.build_bvh(0.01);

                scene.unapply_model_matrices();
            }
        }

        // assign shader to the object
        {
            let mut scene = scene.write().unwrap();
            let shader_list = shader_list.read().unwrap();
            self.get_object_shader()
                .iter()
                .for_each(|(object_name, shader_name)| {
                    let object = scene
                        .get_objects_mut()
                        .find(|object| object.get_object_name() == object_name)
                        .unwrap_or_else(|| panic!("No object with name {} was found", object_name));

                    let shader = shader_list
                        .get_shaders()
                        .find(|shader| shader.get_shader_name() == shader_name)
                        .unwrap_or_else(|| panic!("No shader with name {} was found", shader_name));

                    object.set_path_trace_shader_id(shader.get_shader_id());
                });
        }

        (
            RayTraceParams::new(
                image_width,
                image_height,
                self.get_trace_max_depth()
                    .unwrap_or_else(crate::default_trace_max_depth),
                self.get_samples()
                    .unwrap_or_else(crate::default_samples_per_pixel),
                Arc::try_unwrap(path_trace_camera)
                    .unwrap()
                    .into_inner()
                    .unwrap(),
                Arc::new(RwLock::new(Image::new(1, 1))),
            ),
            scene,
            shader_list,
            texture_list,
            environment,
        )
    }
}
