use rt::bvh::BVHTree;
use rt::glm;
use rt::image::{Image, PPM};
use rt::object::objects::Mesh as MeshObject;
use rt::object::objects::Sphere as SphereObject;
use rt::object::{Object, ObjectDrawData};
use rt::path_trace;
use rt::path_trace::camera::Camera as PathTraceCamera;
use rt::path_trace::camera::CameraDrawData as PathTraceCameraDrawData;
use rt::path_trace::ray::Ray;
use rt::path_trace::shader_list::ShaderList;
use rt::path_trace::traversal_info::{TraversalInfo, TraversalInfoDrawData};
use rt::rasterize::gpu_utils::draw_plane_with_image;
use rt::rasterize::texture::TextureRGBAFloat;
use rt::scene::Scene;
use rt::sphere::Sphere;

extern crate lazy_static;
use rayon::prelude::*;

pub struct RayTraceParams {
    width: usize,
    height: usize,
    trace_max_depth: usize,
    samples_per_pixel: usize,
    camera: PathTraceCamera,
}

impl RayTraceParams {
    pub fn new(
        width: usize,
        height: usize,
        trace_max_depth: usize,
        samples_per_pixel: usize,
        camera: PathTraceCamera,
    ) -> Self {
        Self {
            width,
            height,
            trace_max_depth,
            samples_per_pixel,
            camera,
        }
    }

    /// Get ray trace params's width.
    pub fn get_width(&self) -> usize {
        self.width
    }

    /// Get ray trace params's height.
    pub fn get_height(&self) -> usize {
        self.height
    }

    /// Get ray trace params's trace_max_depth.
    pub fn get_trace_max_depth(&self) -> usize {
        self.trace_max_depth
    }

    /// Get ray trace params's samples_per_pixel.
    pub fn get_samples_per_pixel(&self) -> usize {
        self.samples_per_pixel
    }

    /// Get a reference to the ray trace params's camera.
    pub fn get_camera(&self) -> &PathTraceCamera {
        &self.camera
    }
}

fn ray_trace_scene(
    ray_trace_params: RayTraceParams,
    scene: Arc<RwLock<Scene>>,
    shader_list: Arc<RwLock<ShaderList>>,
    rendered_image: Arc<RwLock<Image>>,
    progress: Arc<RwLock<Progress>>,
    quit_render: Arc<RwLock<bool>>,
) {
    let mut image = Image::new(ray_trace_params.get_width(), ray_trace_params.get_height());
    progress.write().unwrap().reset();

    let progress_previous_update = Arc::new(RwLock::new(Instant::now()));
    let total_number_of_samples = ray_trace_params.get_samples_per_pixel()
        * ray_trace_params.get_width()
        * ray_trace_params.get_height();

    // initialize all pixels to black
    image
        .get_pixels_mut()
        .par_iter_mut()
        .enumerate()
        .for_each(|(_j, row)| {
            row.par_iter_mut().enumerate().for_each(|(_i, pixel)| {
                *pixel = glm::vec3(0.0, 0.0, 0.0);
            });
        });

    // ray trace
    for processed_samples in 0..ray_trace_params.get_samples_per_pixel() {
        if *quit_render.read().unwrap() {
            return;
        }

        let processed_pixels = Arc::new(AtomicUsize::new(0));

        scene.write().unwrap().apply_model_matrices();

        let scene = scene.read().unwrap();
        let shader_list = shader_list.read().unwrap();
        image
            .get_pixels_mut()
            .par_iter_mut()
            .enumerate()
            .for_each(|(j, row)| {
                row.par_iter_mut().enumerate().for_each(|(i, pixel)| {
                    let processed_pixels = processed_pixels.fetch_add(1, Ordering::SeqCst);

                    if progress_previous_update
                        .read()
                        .unwrap()
                        .elapsed()
                        .as_secs_f64()
                        > 0.03
                    {
                        let calculated_progress = (processed_samples
                            * ray_trace_params.get_width()
                            * ray_trace_params.get_height()
                            + processed_pixels)
                            as f64
                            / total_number_of_samples as f64;

                        progress.write().unwrap().set_progress(calculated_progress);

                        *progress_previous_update.write().unwrap() = Instant::now();
                    }

                    let j = ray_trace_params.get_height() - j - 1;

                    // use opengl coords, (0.0, 0.0) is center; (1.0, 1.0) is
                    // top right; (-1.0, -1.0) is bottom left
                    let u = (((i as f64 + rand::random::<f64>())
                        / (ray_trace_params.get_width() - 1) as f64)
                        - 0.5)
                        * 2.0;
                    let v = (((j as f64 + rand::random::<f64>())
                        / (ray_trace_params.get_height() - 1) as f64)
                        - 0.5)
                        * 2.0;

                    let ray = ray_trace_params.get_camera().get_ray(u, v);

                    let (color, _traversal_info) = path_trace::trace_ray(
                        &ray,
                        ray_trace_params.get_camera(),
                        &scene,
                        ray_trace_params.get_trace_max_depth(),
                        &shader_list,
                    );

                    *pixel += color;
                });
            });

        {
            let mut rendered_image = rendered_image.write().unwrap();
            *rendered_image = image.clone();
            rendered_image
                .get_pixels_mut()
                .par_iter_mut()
                .enumerate()
                .for_each(|(_j, row)| {
                    row.par_iter_mut().enumerate().for_each(|(_i, pixel)| {
                        *pixel /= (processed_samples + 1) as f64;
                    });
                });
        }

        {
            let mut progress = progress.write().unwrap();
            progress.set_progress(
                (processed_samples + 1) as f64 / ray_trace_params.get_samples_per_pixel() as f64,
            );
        }
    }

    scene.write().unwrap().unapply_model_matrices();
}

pub enum RayTraceMessage {
    StartRender(RayTraceParams),
    Quit,
}

fn ray_trace_quit_render(
    quit_render: Arc<RwLock<bool>>,
    render_thread_handle: Option<JoinHandle<()>>,
) -> Option<JoinHandle<()>> {
    *quit_render.write().unwrap() = true;
    let render_thread_handle = render_thread_handle.and_then(|join_handle| {
        join_handle.join().unwrap();
        None
    });
    *quit_render.write().unwrap() = false;
    render_thread_handle
}

#[derive(Debug, Clone)]
pub struct Progress {
    progress: f64,
    instant: Instant,
    finished_time: Option<Duration>,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            instant: Instant::now(),
            finished_time: None,
        }
    }

    pub fn get_progress(&self) -> f64 {
        self.progress
    }

    pub fn set_progress(&mut self, prog: f64) {
        if (prog - 1.0).abs() < f64::EPSILON {
            self.finished_time = Some(self.instant.elapsed());
        }
        self.progress = prog;
    }

    pub fn reset(&mut self) {
        self.progress = 0.0;
        self.instant = Instant::now();
        self.finished_time = None;
    }

    pub fn get_elapsed_time(&self) -> f64 {
        if (self.progress - 1.0).abs() < f64::EPSILON {
            self.finished_time.unwrap().as_secs_f64()
        } else {
            self.instant.elapsed().as_secs_f64()
        }
    }

    pub fn get_remaining_time(&self) -> f64 {
        if (self.progress - 1.0).abs() < f64::EPSILON {
            return 0.0;
        }
        let time_diff = self.instant.elapsed().as_secs_f64();

        time_diff / self.progress - self.get_elapsed_time()
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

fn ray_trace_main(
    scene: Arc<RwLock<Scene>>,
    shader_list: Arc<RwLock<ShaderList>>,
    rendered_image: Arc<RwLock<Image>>,
    progress: Arc<RwLock<Progress>>,
    message_receiver: Receiver<RayTraceMessage>,
) {
    let quit_render = Arc::new(RwLock::new(false));
    let mut render_thread_handle: Option<JoinHandle<()>> = None;

    loop {
        let message = message_receiver.recv().unwrap();
        match message {
            RayTraceMessage::StartRender(params) => {
                // quit any previously running ray traces
                ray_trace_quit_render(quit_render.clone(), render_thread_handle);

                let scene = scene.clone();
                let shader_list = shader_list.clone();
                let rendered_image = rendered_image.clone();
                let progress = progress.clone();
                let quit_render = quit_render.clone();
                render_thread_handle = Some(thread::spawn(move || {
                    ray_trace_scene(
                        params,
                        scene,
                        shader_list,
                        rendered_image,
                        progress,
                        quit_render,
                    );
                }));
            }
            RayTraceMessage::Quit => {
                render_thread_handle =
                    ray_trace_quit_render(quit_render.clone(), render_thread_handle);
            }
        }
    }
}

use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use egui::{FontDefinitions, FontFamily, TextStyle};
use egui_glfw::EguiBackend;
use glfw::{Action, Context, Key};

use rt::fps::FPS;
use rt::mesh;
use rt::mesh::MeshUseShader;
use rt::rasterize::camera::Camera as RasterizeCamera;
use rt::rasterize::drawable::Drawable;
use rt::rasterize::gpu_immediate::GPUImmediate;
use rt::rasterize::infinite_grid::{InfiniteGrid, InfiniteGridDrawData};
use rt::rasterize::shader;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    // creating window
    let (mut window, events) = glfw
        .create_window(1280, 720, "RT Rust", glfw::WindowMode::Windowed)
        .expect("ERROR: glfw window creation failed");

    // setup bunch of polling data
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_scroll_polling(true);
    window.set_char_polling(true);
    window.make_current();

    gl::load_with(|symbol| window.get_proc_address(symbol));

    unsafe {
        gl::Disable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::MULTISAMPLE);
    }

    // setup the egui backend
    let mut egui = EguiBackend::new(&mut window, &mut glfw);

    let mut fonts = FontDefinitions::default();
    // larger text
    fonts
        .family_and_size
        .insert(TextStyle::Button, (FontFamily::Proportional, 18.0));
    fonts
        .family_and_size
        .insert(TextStyle::Body, (FontFamily::Proportional, 18.0));
    fonts
        .family_and_size
        .insert(TextStyle::Small, (FontFamily::Proportional, 15.0));
    egui.get_egui_ctx().set_fonts(fonts);

    let mesh = mesh::builtins::get_monkey_subd_00_triangulated();

    let mut camera = RasterizeCamera::new(
        glm::vec3(0.0, 0.0, 3.0),
        glm::vec3(0.0, 1.0, 0.0),
        -90.0,
        0.0,
        45.0,
    );

    let imm = Rc::new(RefCell::new(GPUImmediate::new()));

    shader::builtins::display_uniform_and_attribute_info();

    let mut last_cursor = window.get_cursor_pos();

    let mut fps = FPS::default();

    // TODO(ish): handle drawing bvh and casting rays to the bvh and
    // such again later, right now due to the mesh defined in the
    // scene, it becomes hard to handle it.
    let mut background_color = glm::vec4(0.051, 0.051, 0.051, 1.0);
    let mut draw_bvh = false;
    let mut bvh_draw_level = 0;
    let mut should_cast_bvh_ray = false;
    let mut should_cast_scene_ray = false;
    let mut bvh_color = glm::vec4(0.9, 0.5, 0.2, 1.0);
    let mut bvh_ray_color: glm::DVec4 = glm::vec4(0.2, 0.5, 0.9, 1.0);
    let mut bvh_ray_intersection = Vec::new();
    let mut image_width = 200;
    let mut image_height = 200;
    let mut trace_max_depth = 5;
    let mut samples_per_pixel = 5;
    let mut save_image_location = "test.ppm".to_string();
    let mut ray_traversal_info: Vec<TraversalInfo> = Vec::new();
    let mut ray_to_shoot = (3, 3);
    let mut ray_pixel_start = (
        image_width / ray_to_shoot.0 / 2,
        image_height / ray_to_shoot.1 / 2,
    );
    let mut show_ray_traversal_info = true;
    let mut draw_normal_at_hit_points = true;
    let mut normals_size = 0.4;
    let mut normals_color = glm::vec4(1.0, 1.0, 1.0, 1.0);
    let mut camera_image_alpha_value = 0.0;
    let mut camera_focal_length = 12.0;
    let mut camera_sensor_width = 2.0;
    let mut camera_position = glm::vec3(0.0, 0.0, 10.0);
    let path_trace_progress = Arc::new(RwLock::new(Progress::new()));

    let (shader_list, shader_ids) = {
        let mut shader_list = ShaderList::new();
        let mut shader_ids = Vec::new();

        let id = shader_list.add_shader(Box::new(path_trace::shaders::Lambert::new(
            path_trace::bsdfs::lambert::Lambert::new(glm::vec4(1.0, 1.0, 1.0, 1.0)),
        )));
        shader_ids.push(id);
        let id = shader_list.add_shader(Box::new(path_trace::shaders::Lambert::new(
            path_trace::bsdfs::lambert::Lambert::new(glm::vec4(1.0, 0.0, 0.0, 1.0)),
        )));
        shader_ids.push(id);
        let id = shader_list.add_shader(Box::new(path_trace::shaders::Glossy::new(
            path_trace::bsdfs::glossy::Glossy::new(glm::vec4(1.0, 1.0, 1.0, 1.0)),
        )));
        shader_ids.push(id);
        let id = shader_list.add_shader(Box::new(path_trace::shaders::Emissive::new(
            path_trace::bsdfs::emissive::Emissive::new(glm::vec4(1.0, 0.4, 1.0, 1.0), 5.0),
        )));
        shader_ids.push(id);

        (shader_list, shader_ids)
    };

    let mut scene = Scene::new();
    scene.add_object({
        let mut object = Box::new(SphereObject::new(
            Sphere::new(glm::vec3(0.0, 2.0, -2.0), 0.9),
            glm::vec4(0.0, 0.0, 1.0, 1.0),
            glm::vec4(1.0, 0.0, 0.0, 1.0),
        ));
        object.set_path_trace_shader_id(shader_ids[0]);
        object
    });
    scene.add_object({
        let mut object = Box::new(SphereObject::new(
            Sphere::new(glm::vec3(0.0, -2.0, -2.0), 0.9),
            glm::vec4(0.0, 0.0, 1.0, 1.0),
            glm::vec4(1.0, 0.0, 0.0, 1.0),
        ));
        object.set_path_trace_shader_id(shader_ids[0]);
        object
    });
    scene.add_object({
        let mut object = Box::new(SphereObject::new(
            Sphere::new(glm::vec3(2.0, 0.0, -2.0), 0.9),
            glm::vec4(0.0, 0.0, 1.0, 1.0),
            glm::vec4(1.0, 0.0, 0.0, 1.0),
        ));
        object.set_path_trace_shader_id(shader_ids[0]);
        object
    });
    scene.add_object({
        let mut object = Box::new(SphereObject::new(
            Sphere::new(glm::vec3(-2.0, 0.0, -2.0), 0.9),
            glm::vec4(0.0, 0.0, 1.0, 1.0),
            glm::vec4(1.0, 0.0, 0.0, 1.0),
        ));
        object.set_path_trace_shader_id(shader_ids[0]);
        object
    });
    scene.add_object({
        let mut object = Box::new(MeshObject::new(
            mesh.clone(),
            MeshUseShader::DirectionalLight,
            draw_bvh,
            bvh_draw_level,
            bvh_color,
        ));
        object.set_path_trace_shader_id(shader_ids[1]);
        object
    });
    // scene.add_object({
    //     let mut object = Box::new(SphereObject::new(
    //         Sphere::new(glm::vec3(0.0, 0.0, -2.0), 0.9),
    //         glm::vec4(0.0, 0.0, 1.0, 1.0),
    //         glm::vec4(1.0, 0.0, 0.0, 1.0),
    //     ));
    //     object.set_path_trace_shader_id(shader_ids[3]);
    //     object
    // });
    // scene.add_object({
    //     let mut object = Box::new(MeshObject::new(
    //         mesh::builtins::get_plane_subd_00().clone(),
    //         MeshUseShader::DirectionalLight,
    //         draw_bvh,
    //         bvh_draw_level,
    //         bvh_color,
    //     ));
    //     object.set_path_trace_shader_id(shader_ids[2]);
    //     object.set_model_matrix(glm::rotate_x(
    //         &glm::scale(
    //             &glm::translate(&glm::identity(), &glm::vec3(0.0, 0.0, 5.0)),
    //             &glm::vec3(5.0, 5.0, 5.0),
    //         ),
    //         glm::radians(&glm::vec1(90.0))[0],
    //     ));
    //     object
    // });

    scene.get_objects_mut().iter_mut().for_each(|object| {
        if object.get_model_matrix().is_none() {
            object.set_model_matrix(glm::translate(&glm::identity(), &glm::vec3(0.0, 0.0, -2.0)));
        }
    });

    let scene = Arc::new(RwLock::new(scene));
    let shader_list = Arc::new(RwLock::new(shader_list));

    let infinite_grid = InfiniteGrid::default();

    let rendered_image = Arc::new(RwLock::new(Image::new(100, 100)));

    // Spawn the main ray tracing thread
    let (ray_trace_thread_sender, ray_trace_thread_receiver) = mpsc::channel();
    {
        let scene = scene.clone();
        let shader_list = shader_list.clone();
        let rendered_image = rendered_image.clone();
        let path_trace_progress = path_trace_progress.clone();
        thread::spawn(move || {
            ray_trace_main(
                scene,
                shader_list,
                rendered_image,
                path_trace_progress,
                ray_trace_thread_receiver,
            );
        });
    }

    while !window.should_close() {
        glfw.poll_events();

        glfw::flush_messages(&events).for_each(|(_, event)| {
            egui.handle_event(&event, &window);

            handle_window_event(
                &event,
                &mut window,
                &mut camera,
                &mut should_cast_bvh_ray,
                &mut should_cast_scene_ray,
                &mut last_cursor,
            );
        });

        let path_trace_camera = {
            let aspect_ratio = image_width as f64 / image_height as f64;
            let camera_sensor_height = camera_sensor_width / aspect_ratio;
            PathTraceCamera::new(
                camera_sensor_height,
                aspect_ratio,
                camera_focal_length,
                camera_position,
            )
        };

        let mut rendered_texture = TextureRGBAFloat::from_image(&rendered_image.read().unwrap());

        unsafe {
            let background_color: glm::Vec4 = glm::convert(background_color);
            gl::ClearColor(
                background_color[0],
                background_color[1],
                background_color[2],
                background_color[3],
            );
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let (window_width, window_height) = window.get_size();
        let (window_width, window_height): (usize, usize) = (
            window_width.try_into().unwrap(),
            window_height.try_into().unwrap(),
        );

        // Shader stuff
        shader::builtins::setup_shaders(&camera, window_width, window_height);

        unsafe {
            gl::Disable(gl::BLEND);
        }

        scene
            .read()
            .unwrap()
            .draw(&mut ObjectDrawData::new(imm.clone()))
            .unwrap();

        if show_ray_traversal_info {
            ray_traversal_info.iter().for_each(|info| {
                info.draw(&mut TraversalInfoDrawData::new(
                    imm.clone(),
                    draw_normal_at_hit_points,
                    normals_size,
                    normals_color,
                ))
                .unwrap();
            });
        }

        draw_plane_with_image(
            &glm::vec3(2.0, image_height as f64 / 1000.0, 0.0),
            &glm::vec3(image_width as f64 / 500.0, 2.0, image_height as f64 / 500.0),
            &glm::vec3(0.0, 0.0, 1.0),
            &mut rendered_texture,
            1.0,
            &mut imm.borrow_mut(),
        );

        if should_cast_bvh_ray {
            let ray_direction = camera.get_raycast_direction(
                last_cursor.0,
                last_cursor.1,
                window_width,
                window_height,
            );

            let bvh: &BVHTree<usize> = mesh.get_bvh().as_ref().unwrap();

            if let Some(ray_hit_info) = bvh.ray_cast(
                camera.get_position(),
                ray_direction,
                None::<&fn((&glm::DVec3, &glm::DVec3), _) -> Option<rt::bvh::RayHitData<_>>>,
            ) {
                bvh_ray_intersection.push((camera.get_position(), ray_hit_info));
            }

            should_cast_bvh_ray = false;
        }

        if should_cast_scene_ray {
            let ray_direction = camera.get_raycast_direction(
                last_cursor.0,
                last_cursor.1,
                window_width,
                window_height,
            );

            scene.write().unwrap().apply_model_matrices();

            // trace ray into scene from the rasterizer camera
            // position to get the first hitpoint
            let (_color, traversal_info) = path_trace::trace_ray(
                &Ray::new(camera.get_position(), ray_direction),
                &path_trace_camera,
                &scene.read().unwrap(),
                1,
                &shader_list.read().unwrap(),
            );

            // generate the new ray from the path_trace_camera's
            // position towards the first hitpoint
            let ray_direction = if let Some(hit_point) = traversal_info.get_traversal()[0].get_co()
            {
                (hit_point - path_trace_camera.get_origin()).normalize()
            } else {
                (traversal_info.get_traversal()[0].get_ray().at(1000.0)
                    - path_trace_camera.get_origin())
                .normalize()
            };

            let (_color, traversal_info) = path_trace::trace_ray(
                &Ray::new(*path_trace_camera.get_origin(), ray_direction),
                &path_trace_camera,
                &scene.read().unwrap(),
                trace_max_depth,
                &shader_list.read().unwrap(),
            );

            scene.write().unwrap().unapply_model_matrices();

            ray_traversal_info.clear();
            ray_traversal_info.push(traversal_info);

            should_cast_scene_ray = false;
        }

        {
            if !bvh_ray_intersection.is_empty() {
                let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
                    .as_ref()
                    .unwrap();
                smooth_color_3d_shader.use_shader();
                smooth_color_3d_shader.set_mat4("model\0", &glm::identity());

                let mut imm = imm.borrow_mut();

                let format = imm.get_cleared_vertex_format();
                let pos_attr = format.add_attribute(
                    "in_pos\0".to_string(),
                    rt::rasterize::gpu_immediate::GPUVertCompType::F32,
                    3,
                    rt::rasterize::gpu_immediate::GPUVertFetchMode::Float,
                );
                let color_attr = format.add_attribute(
                    "in_color\0".to_string(),
                    rt::rasterize::gpu_immediate::GPUVertCompType::F32,
                    4,
                    rt::rasterize::gpu_immediate::GPUVertFetchMode::Float,
                );

                imm.begin(
                    rt::rasterize::gpu_immediate::GPUPrimType::Lines,
                    bvh_ray_intersection.len() * 2,
                    smooth_color_3d_shader,
                );

                let bvh_ray_color: glm::Vec4 = glm::convert(bvh_ray_color);

                bvh_ray_intersection.iter().for_each(|(pos, ray_hit_info)| {
                    let p1: glm::Vec3 = glm::convert(*pos);
                    let p2: glm::Vec3 = glm::convert(ray_hit_info.data.as_ref().unwrap().co);

                    imm.attr_4f(
                        color_attr,
                        bvh_ray_color[0],
                        bvh_ray_color[1],
                        bvh_ray_color[2],
                        bvh_ray_color[3],
                    );
                    imm.vertex_3f(pos_attr, p1[0], p1[1], p1[2]);
                    imm.attr_4f(
                        color_attr,
                        bvh_ray_color[0],
                        bvh_ray_color[1],
                        bvh_ray_color[2],
                        bvh_ray_color[3],
                    );
                    imm.vertex_3f(pos_attr, p2[0], p2[1], p2[2]);
                });

                imm.end();
            }
        }

        // Keep meshes that have shaders that need alpha channel
        // (blending) below this and handle it properly
        {
            unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            }

            let rc_refcell_image = Rc::new(RefCell::new(rendered_texture));
            path_trace_camera
                .draw(&mut PathTraceCameraDrawData::new(
                    imm.clone(),
                    Some(rc_refcell_image.clone()),
                    camera_image_alpha_value,
                ))
                .unwrap();
            rendered_texture = match Rc::try_unwrap(rc_refcell_image) {
                Ok(refcell_image) => refcell_image.into_inner(),
                Err(_) => unreachable!("rc_refcell_image should not be in a borrowed state now"),
            };

            infinite_grid
                .draw(&mut InfiniteGridDrawData::new(imm.clone()))
                .unwrap();

            // GUI starts
            {
                egui.begin_frame(&window, &mut glfw);
                egui::Window::new("Hello world!").show(egui.get_egui_ctx(), |ui| {
                    egui::ScrollArea::auto_sized().show(ui, |ui| {
                        ui.label(format!("fps: {:.2}", fps.update_and_get(Some(60.0))));
                        ui.add({
                            let path_trace_progress = &*path_trace_progress.read().unwrap();
                            egui::ProgressBar::new(path_trace_progress.get_progress() as _)
                                .text(format!(
                                    "Path Trace Progress: {:.2}%",
                                    path_trace_progress.get_progress() * 100.0
                                ))
                                .animate(true)
                        });
                        ui.label(format!(
                            "Time Elapsed (in secs) {:.2}",
                            path_trace_progress.read().unwrap().get_elapsed_time()
                        ));
                        ui.label(format!(
                            "Time Left (in secs) {:.2}",
                            path_trace_progress.read().unwrap().get_remaining_time()
                        ));

                        color_edit_button_dvec4(ui, "Background Color", &mut background_color);

                        ui.checkbox(&mut draw_bvh, "Draw BVH");
                        ui.add(
                            egui::Slider::new(&mut bvh_draw_level, 0..=15).text("BVH Draw Level"),
                        );
                        color_edit_button_dvec4(ui, "BVH Color", &mut bvh_color);
                        color_edit_button_dvec4(ui, "BVH Ray Color", &mut bvh_ray_color);

                        if ui.button("Delete Rays").clicked() {
                            bvh_ray_intersection.clear();
                        }

                        ui.separator();

                        ui.add(
                            egui::Slider::new(&mut camera_image_alpha_value, 0.0..=1.0)
                                .clamp_to_range(true)
                                .text("Camera Image Alpha"),
                        );

                        ui.add(
                            egui::Slider::new(&mut camera_sensor_width, 0.0..=36.0)
                                .text("Camera Sensor Width"),
                        );

                        ui.add(
                            egui::Slider::new(&mut camera_focal_length, 0.0..=15.0)
                                .text("Camera Focal Length"),
                        );

                        ui.label("Camera Position");
                        ui.add(egui::Slider::new(&mut camera_position[0], -10.0..=10.0).text("x"));
                        ui.add(egui::Slider::new(&mut camera_position[1], -10.0..=10.0).text("y"));
                        ui.add(egui::Slider::new(&mut camera_position[2], -10.0..=10.0).text("z"));

                        ui.separator();

                        ui.add(egui::Slider::new(&mut image_width, 1..=1000).text("Image Width"));
                        if image_width == 0 {
                            image_width = 1;
                        }
                        ui.add(egui::Slider::new(&mut image_height, 1..=1000).text("Image Height"));
                        if image_height == 0 {
                            image_height = 1;
                        }
                        ui.add(
                            egui::Slider::new(&mut trace_max_depth, 1..=10).text("Trace Max Depth"),
                        );
                        ui.add(
                            egui::Slider::new(&mut samples_per_pixel, 1..=10)
                                .text("Samples Per Pixel"),
                        );

                        ui.horizontal(|ui| {
                            if ui.button("Ray Trace Scene").clicked() {
                                ray_trace_thread_sender
                                    .send(RayTraceMessage::StartRender(RayTraceParams::new(
                                        image_width,
                                        image_height,
                                        trace_max_depth,
                                        samples_per_pixel,
                                        path_trace_camera.clone(),
                                    )))
                                    .unwrap();
                            }

                            if ui.button("Quit Render").clicked() {
                                ray_trace_thread_sender.send(RayTraceMessage::Quit).unwrap();
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Save Location");
                            ui.text_edit_singleline(&mut save_image_location);
                        });

                        if ui.button("Save Ray Traced Image").clicked() {
                            let image = Image::from_texture_rgba_float(&rendered_texture);
                            PPM::new(&image)
                                .write_to_file(&save_image_location)
                                .unwrap();
                        }

                        ui.separator();

                        ui.label("Rays to Shoot");
                        ui.add(
                            egui::Slider::new(&mut ray_to_shoot.0, 1..=image_width)
                                .logarithmic(true)
                                .clamp_to_range(true)
                                .text("x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut ray_to_shoot.1, 1..=image_height)
                                .logarithmic(true)
                                .clamp_to_range(true)
                                .text("y"),
                        );
                        ui.label("Ray Pixel Start");
                        ui.add(
                            egui::Slider::new(
                                &mut ray_pixel_start.0,
                                0..=(image_width / ray_to_shoot.0) - 1,
                            )
                            .clamp_to_range(true)
                            .text("x"),
                        );
                        ui.add(
                            egui::Slider::new(
                                &mut ray_pixel_start.1,
                                0..=(image_height / ray_to_shoot.1) - 1,
                            )
                            .clamp_to_range(true)
                            .text("y"),
                        );
                        ui.checkbox(&mut show_ray_traversal_info, "Show Ray Traversal Info");

                        ui.checkbox(&mut draw_normal_at_hit_points, "Draw Normal at Hit Points");
                        ui.add(
                            egui::Slider::new(&mut normals_size, 0.0..=2.0).text("Normals Size"),
                        );
                        color_edit_button_dvec4(ui, "Normals Color", &mut normals_color);

                        if ui.button("Trace Rays").clicked() {
                            scene.write().unwrap().apply_model_matrices();

                            ray_traversal_info.clear();

                            for i in 0..ray_to_shoot.0 {
                                for j in 0..ray_to_shoot.1 {
                                    let i = i * (image_width / ray_to_shoot.0) + ray_pixel_start.0;
                                    let j = j * (image_height / ray_to_shoot.1) + ray_pixel_start.1;
                                    // use opengl coords, (0.0, 0.0) is center; (1.0, 1.0) is
                                    // top right; (-1.0, -1.0) is bottom left
                                    let u = (((i as f64 + rand::random::<f64>())
                                        / (image_width - 1) as f64)
                                        - 0.5)
                                        * 2.0;
                                    let v = (((j as f64 + rand::random::<f64>())
                                        / (image_height - 1) as f64)
                                        - 0.5)
                                        * 2.0;

                                    let ray = path_trace_camera.get_ray(u, v);

                                    let (_color, traversal_info) = path_trace::trace_ray(
                                        &ray,
                                        &path_trace_camera,
                                        &scene.read().unwrap(),
                                        trace_max_depth,
                                        &shader_list.read().unwrap(),
                                    );
                                    ray_traversal_info.push(traversal_info);
                                }
                            }

                            scene.write().unwrap().unapply_model_matrices();
                        }
                    });
                });
                let _output = egui.end_frame(glm::vec2(window_width as _, window_height as _));
            }
            // GUI ends
        }

        // Swap front and back buffers
        window.swap_buffers();
    }
}

fn handle_window_event(
    event: &glfw::WindowEvent,
    window: &mut glfw::Window,
    camera: &mut RasterizeCamera,
    should_cast_bvh_ray: &mut bool,
    should_cast_scene_ray: &mut bool,
    last_cursor: &mut (f64, f64),
) {
    let cursor = window.get_cursor_pos();
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true);
        }

        glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
            gl::Viewport(0, 0, *width, *height);
        },
        glfw::WindowEvent::Scroll(_, scroll_y) => {
            camera.zoom(*scroll_y);
        }
        _ => {}
    };

    let (window_width, window_height) = window.get_size();
    let (window_width, window_height): (usize, usize) = (
        window_width.try_into().unwrap(),
        window_height.try_into().unwrap(),
    );

    if window.get_mouse_button(glfw::MouseButtonMiddle) == glfw::Action::Press {
        if window.get_key(glfw::Key::LeftShift) == glfw::Action::Press {
            camera.pan(
                last_cursor.0,
                last_cursor.1,
                cursor.0,
                cursor.1,
                1.0,
                window_width,
                window_height,
            );
        } else if window.get_key(glfw::Key::LeftControl) == glfw::Action::Press {
            camera.move_forward(last_cursor.1, cursor.1, window_height);
        } else {
            camera.rotate_wrt_camera_origin(
                last_cursor.0,
                last_cursor.1,
                cursor.0,
                cursor.1,
                0.1,
                false,
            );
        }
    }

    if window.get_mouse_button(glfw::MouseButtonLeft) == glfw::Action::Press
        && window.get_key(glfw::Key::LeftControl) == glfw::Action::Press
    {
        *should_cast_bvh_ray = true;
    }

    if window.get_mouse_button(glfw::MouseButtonLeft) == glfw::Action::Press
        && window.get_key(glfw::Key::LeftAlt) == glfw::Action::Press
    {
        *should_cast_scene_ray = true;
    }

    *last_cursor = cursor;
}

fn color_edit_dvec4(ui: &mut egui::Ui, color: &mut glm::DVec4) {
    let mut color_egui = egui::Color32::from_rgba_premultiplied(
        (color[0] * 255.0) as _,
        (color[1] * 255.0) as _,
        (color[2] * 255.0) as _,
        (color[3] * 255.0) as _,
    );
    egui::color_picker::color_edit_button_srgba(
        ui,
        &mut color_egui,
        egui::color_picker::Alpha::BlendOrAdditive,
    );
    *color = glm::vec4(
        color_egui.r() as f64 / 255.0,
        color_egui.g() as f64 / 255.0,
        color_egui.b() as f64 / 255.0,
        color_egui.a() as f64 / 255.0,
    );
}

fn color_edit_button_dvec4(ui: &mut egui::Ui, text: &str, color: &mut glm::DVec4) {
    ui.horizontal(|ui| {
        ui.label(text);
        color_edit_dvec4(ui, color);
    });
}
