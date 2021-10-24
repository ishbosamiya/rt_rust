use glm::Scalar;
use rfd::FileDialog;
use rt::image::{Image, PPM};
use rt::object::objects::Mesh as MeshObject;
use rt::object::objects::Sphere as SphereObject;
use rt::object::{Object, ObjectDrawData};
use rt::path_trace::camera::Camera as PathTraceCamera;
use rt::path_trace::camera::CameraDrawData as PathTraceCameraDrawData;
use rt::path_trace::intersectable::Intersectable;
use rt::path_trace::ray::Ray;
use rt::path_trace::shader_list::{ShaderID, ShaderList};
use rt::path_trace::traversal_info::{TraversalInfo, TraversalInfoDrawData};
use rt::path_trace::{self, RayTraceMessage, RayTraceParams};
use rt::progress::Progress;
use rt::rasterize::gpu_utils::draw_plane_with_image;
use rt::rasterize::texture::TextureRGBAFloat;
use rt::scene::Scene;
use rt::sphere::Sphere;
use rt::ui::DrawUI;
use rt::{glm, ui};

extern crate lazy_static;

use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

use egui::{FontDefinitions, FontFamily, TextStyle};
use egui_glfw::EguiBackend;
use glfw::{Action, Context, Key};
use serde::{Deserialize, Serialize};

use rt::fps::FPS;
use rt::mesh;
use rt::mesh::MeshUseShader;
use rt::rasterize::camera::Camera as RasterizeCamera;
use rt::rasterize::drawable::Drawable;
use rt::rasterize::gpu_immediate::GPUImmediate;
use rt::rasterize::infinite_grid::{InfiniteGrid, InfiniteGridDrawData};
use rt::rasterize::shader;

#[derive(Debug, Serialize, Deserialize)]
struct File {
    scene: Arc<RwLock<Scene>>,
    shader_list: Arc<RwLock<ShaderList>>,
    path_trace_camera: Arc<RwLock<PathTraceCamera>>,
}

impl File {
    fn new(
        scene: Arc<RwLock<Scene>>,
        shader_list: Arc<RwLock<ShaderList>>,
        path_trace_camera: Arc<RwLock<PathTraceCamera>>,
    ) -> Self {
        Self {
            scene,
            shader_list,
            path_trace_camera,
        }
    }
}

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

    let mut use_top_panel = true;
    let mut use_bottom_panel = true;
    let mut use_left_panel = true;
    let mut use_right_panel = true;

    let mut background_color = glm::vec4(0.051, 0.051, 0.051, 1.0);
    let mut should_cast_scene_ray = false;
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
    let mut camera_use_depth_for_image = true;
    let mut selected_shader: Option<ShaderID> = None;
    let mut end_ray_depth: usize = trace_max_depth;
    let mut start_ray_depth: usize = 1;

    let path_trace_camera = {
        let camera_focal_length = 12.0;
        let camera_sensor_width = 2.0;
        let camera_position = glm::vec3(0.0, 0.0, 10.0);
        let aspect_ratio = image_width as f64 / image_height as f64;
        let camera_sensor_height = camera_sensor_width / aspect_ratio;
        PathTraceCamera::new(
            camera_sensor_height,
            aspect_ratio,
            camera_focal_length,
            camera_position,
        )
    };

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
            MeshUseShader::DirectionalLight {
                color: glm::vec3(0.3, 0.2, 0.7),
            },
            None,
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
    //         None,
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

    // build bvh
    {
        scene.apply_model_matrices();

        scene.build_bvh(0.01);

        scene.unapply_model_matrices();
    }

    let scene = Arc::new(RwLock::new(scene));
    let shader_list = Arc::new(RwLock::new(shader_list));
    let path_trace_camera = Arc::new(RwLock::new(path_trace_camera));

    let infinite_grid = InfiniteGrid::default();

    let rendered_image = Arc::new(RwLock::new(Image::new(100, 100)));

    // Spawn the main ray tracing thread
    let (ray_trace_thread_sender, ray_trace_thread_receiver) = mpsc::channel();
    let ray_trace_main_thread_handle = {
        let scene = scene.clone();
        let shader_list = shader_list.clone();
        let camera = path_trace_camera.clone();
        let rendered_image = rendered_image.clone();
        let path_trace_progress = path_trace_progress.clone();
        thread::spawn(move || {
            path_trace::ray_trace_main(
                scene,
                shader_list,
                camera,
                rendered_image,
                path_trace_progress,
                ray_trace_thread_receiver,
            );
        })
    };

    while !window.should_close() {
        glfw.poll_events();

        glfw::flush_messages(&events).for_each(|(_, event)| {
            egui.handle_event(&event, &window);

            handle_window_event(
                &event,
                &mut window,
                &mut camera,
                &path_trace_camera.read().unwrap(),
                &mut should_cast_scene_ray,
                &mut use_top_panel,
                &mut use_bottom_panel,
                &mut use_left_panel,
                &mut use_right_panel,
                &mut last_cursor,
            );
        });

        // TODO: need to fix this performance bottleneck. Should
        // update rendered_texture only when rendered_image has
        // changed
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

        let (window_width, window_height): (usize, usize) = {
            let (window_width, window_height) = window.get_size();
            (
                window_width.try_into().unwrap(),
                window_height.try_into().unwrap(),
            )
        };
        let (framebuffer_width, framebuffer_height): (usize, usize) = {
            let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
            (
                framebuffer_width.try_into().unwrap(),
                framebuffer_height.try_into().unwrap(),
            )
        };

        // Shader stuff
        shader::builtins::setup_shaders(&camera, window_width, window_height);

        unsafe {
            gl::Disable(gl::BLEND);
        }

        // drawing the scene
        scene
            .read()
            .unwrap()
            .draw(&mut ObjectDrawData::new(imm.clone()))
            .unwrap();

        // drawing ray traversal info if needed
        if show_ray_traversal_info {
            ray_traversal_info.iter().for_each(|info| {
                info.draw(&mut TraversalInfoDrawData::new(
                    imm.clone(),
                    draw_normal_at_hit_points,
                    normals_size,
                    normals_color,
                    start_ray_depth,
                    end_ray_depth,
                ))
                .unwrap();
            });
        }

        // drawing the rendered image at some location
        draw_plane_with_image(
            &glm::vec3(2.0, image_height as f64 / 1000.0, 0.0),
            &glm::vec3(image_width as f64 / 500.0, 2.0, image_height as f64 / 500.0),
            &glm::vec3(0.0, 0.0, 1.0),
            &mut rendered_texture,
            1.0,
            &mut imm.borrow_mut(),
        );

        // handle casting ray into the scene
        if should_cast_scene_ray {
            let ray_direction = camera.get_raycast_direction(
                last_cursor.0,
                last_cursor.1,
                window_width,
                window_height,
            );

            scene.write().unwrap().apply_model_matrices();

            let path_trace_camera = path_trace_camera.read().unwrap();

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

        if window.get_mouse_button(glfw::MouseButtonLeft) == glfw::Action::Press {
            if let Some(shader_id) = selected_shader {
                let ray_direction = camera.get_raycast_direction(
                    last_cursor.0,
                    last_cursor.1,
                    window_width,
                    window_height,
                );
                let mut scene = scene.write().unwrap();
                scene.apply_model_matrices();
                if let Some(hit_info) = scene.hit(
                    &Ray::new(camera.get_position(), ray_direction),
                    0.01,
                    1000.0,
                ) {
                    let object_id = hit_info.get_object_id().unwrap();
                    scene.get_objects_mut().iter_mut().for_each(|object| {
                        if object_id == object.get_object_id() {
                            object.set_path_trace_shader_id(shader_id);
                            selected_shader = None;
                        }
                    });
                }
                scene.unapply_model_matrices();
            }
        }

        // Keep meshes that have shaders that need alpha channel
        // (blending) below this and handle it properly
        {
            unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            }

            // drawing the camera
            let rc_refcell_image = Rc::new(RefCell::new(rendered_texture));
            path_trace_camera
                .read()
                .unwrap()
                .draw(&mut PathTraceCameraDrawData::new(
                    imm.clone(),
                    Some(rc_refcell_image.clone()),
                    camera_image_alpha_value,
                    camera_use_depth_for_image,
                ))
                .unwrap();
            rendered_texture = match Rc::try_unwrap(rc_refcell_image) {
                Ok(refcell_image) => refcell_image.into_inner(),
                Err(_) => unreachable!("rc_refcell_image should not be in a borrowed state now"),
            };

            // drawing the infinite grid
            infinite_grid
                .draw(&mut InfiniteGridDrawData::new(imm.clone()))
                .unwrap();

            // TODO(ish): shift the gui code to be before the rest, so
            // that the viewport and it's dimensions are set correctly
            // for later use.

            // GUI starts
            {
                // set the opengl viewport for the full frame buffer
                // for correct GUI element drawing
                unsafe {
                    gl::Viewport(
                        0,
                        0,
                        framebuffer_width.try_into().unwrap(),
                        framebuffer_height.try_into().unwrap(),
                    );
                }

                egui.begin_frame(&window, &mut glfw);

                // Draw top, right, bottom and left panels, the order
                // matters since it goes from outermost to innermost.

                let top_panel_response = if use_top_panel {
                    let response = egui::TopBottomPanel::top("Top Panel")
                        .resizable(true)
                        .show(egui.get_egui_ctx(), |_ui| {})
                        .response;
                    Some(response)
                } else {
                    None
                };

                let right_panel_response = if use_right_panel {
                    let response = egui::SidePanel::right("Right Side Panel")
                        .min_width(0.1 * window_width as f32)
                        .resizable(true)
                        .show(egui.get_egui_ctx(), |ui| {
                            egui::ScrollArea::auto_sized().show(ui, |ui| {
                                shader_list.read().unwrap().draw_ui(ui);
                                if let Ok(mut shader_list) = shader_list.try_write() {
                                    shader_list.draw_ui_mut(ui);
                                    selected_shader = *shader_list.get_selected_shader();
                                } else {
                                    ui.label(
                                        "Shaders are currently in use, cannot edit the shaders",
                                    );
                                }
                            });
                        })
                        .response;
                    Some(response)
                } else {
                    None
                };

                let bottom_panel_response = if use_bottom_panel {
                    let response = egui::TopBottomPanel::bottom("Bottom Panel")
                        .resizable(true)
                        .show(egui.get_egui_ctx(), |_ui| {})
                        .response;
                    Some(response)
                } else {
                    None
                };

                let left_panel_response = if use_left_panel {
                    let response = egui::SidePanel::left("Left Side Panel")
                        .min_width(0.1 * window_width as f32)
                        .resizable(true)
                        .show(egui.get_egui_ctx(), |ui| {
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

                                if ui.button("Save File").clicked() {
                                    let file = File::new(
                                        scene.clone(),
                                        shader_list.clone(),
                                        path_trace_camera.clone(),
                                    );
                                    let file_serialized = serde_json::to_string(&file).unwrap();
                                    if let Some(path) = FileDialog::new()
                                        .add_filter("RT", &["rt"])
                                        .add_filter("Any", &["*"])
                                        .set_directory(".")
                                        .set_file_name("untitled.rt")
                                        .save_file()
                                    {
                                        std::fs::write(path, file_serialized).unwrap();
                                    }
                                }
                                if ui.button("Load File").clicked() {
                                    if let Some(path) = FileDialog::new()
                                        .add_filter("RT", &["rt"])
                                        .add_filter("Any", &["*"])
                                        .set_directory(".")
                                        .pick_file()
                                    {
                                        let json = String::from_utf8(std::fs::read(path).unwrap())
                                            .unwrap();
                                        let file: File = serde_json::from_str(&json).unwrap();
                                        *scene.write().unwrap() = Arc::try_unwrap(file.scene)
                                            .unwrap()
                                            .into_inner()
                                            .unwrap();
                                        *shader_list.write().unwrap() =
                                            Arc::try_unwrap(file.shader_list)
                                                .unwrap()
                                                .into_inner()
                                                .unwrap();
                                        *path_trace_camera.write().unwrap() =
                                            Arc::try_unwrap(file.path_trace_camera)
                                                .unwrap()
                                                .into_inner()
                                                .unwrap();
                                    }
                                }

                                ui::color_edit_button_dvec4(
                                    ui,
                                    "Background Color",
                                    &mut background_color,
                                );

                                ui.separator();

                                ui.add(
                                    egui::Slider::new(&mut camera_image_alpha_value, 0.0..=1.0)
                                        .clamp_to_range(true)
                                        .text("Camera Image Alpha"),
                                );

                                ui.checkbox(&mut camera_use_depth_for_image, "Use Depth for Image");

                                let camera_sensor_width = {
                                    let mut camera_sensor_width =
                                        path_trace_camera.read().unwrap().get_sensor_width();
                                    ui.add(
                                        egui::Slider::new(&mut camera_sensor_width, 0.0..=36.0)
                                            .text("Camera Sensor Width"),
                                    );
                                    camera_sensor_width
                                };

                                let camera_focal_length = {
                                    let mut camera_focal_length =
                                        path_trace_camera.read().unwrap().get_focal_length();
                                    ui.add(
                                        egui::Slider::new(&mut camera_focal_length, 0.0..=15.0)
                                            .text("Camera Focal Length"),
                                    );
                                    camera_focal_length
                                };

                                let camera_position = {
                                    let mut camera_position =
                                        *path_trace_camera.read().unwrap().get_origin();
                                    ui.label("Camera Position");
                                    ui.add(
                                        egui::Slider::new(&mut camera_position[0], -10.0..=10.0)
                                            .text("x"),
                                    );
                                    ui.add(
                                        egui::Slider::new(&mut camera_position[1], -10.0..=10.0)
                                            .text("y"),
                                    );
                                    ui.add(
                                        egui::Slider::new(&mut camera_position[2], -10.0..=10.0)
                                            .text("z"),
                                    );
                                    camera_position
                                };

                                if let Ok(mut path_trace_camera) = path_trace_camera.try_write() {
                                    path_trace_camera.change_sensor_width(camera_sensor_width);
                                    path_trace_camera.change_aspect_ratio(
                                        image_width as f64 / image_height as f64,
                                    );
                                    path_trace_camera.change_focal_length(camera_focal_length);
                                    path_trace_camera.change_origin(camera_position);
                                }

                                ui.separator();

                                ui.add(
                                    egui::Slider::new(&mut image_width, 1..=1000)
                                        .text("Image Width"),
                                );
                                if image_width == 0 {
                                    image_width = 1;
                                }
                                ui.add(
                                    egui::Slider::new(&mut image_height, 1..=1000)
                                        .text("Image Height"),
                                );
                                if image_height == 0 {
                                    image_height = 1;
                                }
                                ui.add(
                                    egui::Slider::new(&mut trace_max_depth, 1..=10)
                                        .text("Trace Max Depth"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut samples_per_pixel, 1..=10)
                                        .text("Samples Per Pixel"),
                                );

                                ui.horizontal(|ui| {
                                    if ui.button("Ray Trace Scene").clicked() {
                                        ray_trace_thread_sender
                                            .send(RayTraceMessage::StartRender(
                                                RayTraceParams::new(
                                                    image_width,
                                                    image_height,
                                                    trace_max_depth,
                                                    samples_per_pixel,
                                                ),
                                            ))
                                            .unwrap();
                                    }

                                    if ui.button("Stop Render").clicked() {
                                        ray_trace_thread_sender
                                            .send(RayTraceMessage::StopRender)
                                            .unwrap();
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
                                ui.checkbox(
                                    &mut show_ray_traversal_info,
                                    "Show Ray Traversal Info",
                                );

                                ui.add(
                                    egui::Slider::new(&mut start_ray_depth, 1..=end_ray_depth)
                                        .clamp_to_range(true)
                                        .text("Start Ray Depth"),
                                );

                                ui.add(
                                    egui::Slider::new(
                                        &mut end_ray_depth,
                                        start_ray_depth..=trace_max_depth,
                                    )
                                    .clamp_to_range(true)
                                    .text("End Ray Depth"),
                                );

                                ui.checkbox(
                                    &mut draw_normal_at_hit_points,
                                    "Draw Normal at Hit Points",
                                );
                                ui.add(
                                    egui::Slider::new(&mut normals_size, 0.0..=2.0)
                                        .text("Normals Size"),
                                );
                                ui::color_edit_button_dvec4(
                                    ui,
                                    "Normals Color",
                                    &mut normals_color,
                                );

                                if ui.button("Trace Rays").clicked() {
                                    scene.write().unwrap().apply_model_matrices();

                                    let path_trace_camera = path_trace_camera.read().unwrap();

                                    ray_traversal_info.clear();

                                    for i in 0..ray_to_shoot.0 {
                                        for j in 0..ray_to_shoot.1 {
                                            let i = i * (image_width / ray_to_shoot.0)
                                                + ray_pixel_start.0;
                                            let j = j * (image_height / ray_to_shoot.1)
                                                + ray_pixel_start.1;
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
                        })
                        .response;
                    Some(response)
                } else {
                    None
                };

                let viewport = {
                    let mut width = framebuffer_width;
                    let mut height = framebuffer_height;

                    if let Some(top_panel_response) = top_panel_response {
                        height -= top_panel_response.rect.size().y as usize;
                    }
                    let viewport_start_y =
                        if let Some(bottom_panel_response) = bottom_panel_response {
                            height -= bottom_panel_response.rect.size().y as usize;
                            bottom_panel_response.rect.size().y as usize
                        } else {
                            0
                        };
                    let viewport_start_x = if let Some(left_panel_response) = left_panel_response {
                        width -= left_panel_response.rect.size().x as usize;
                        left_panel_response.rect.size().x as usize
                    } else {
                        0
                    };
                    if let Some(right_panel_response) = right_panel_response {
                        width -= right_panel_response.rect.size().x as usize;
                    }

                    (viewport_start_x, viewport_start_y, width, height)
                };

                egui::Window::new("Camera Data")
                    .open(&mut false)
                    .collapsible(true)
                    .show(egui.get_egui_ctx(), |ui| {
                        egui::ScrollArea::auto_sized().show(ui, |ui| {
                            ui.label(format!(
                                "position: {}",
                                vec_to_string(&camera.get_position())
                            ));
                            ui.label(format!("front: {}", vec_to_string(&camera.get_front())));
                            ui.label(format!("up: {}", vec_to_string(&camera.get_up())));
                            ui.label(format!("right: {}", vec_to_string(&camera.get_right())));
                            ui.label(format!(
                                "world_up: {}",
                                vec_to_string(camera.get_world_up())
                            ));
                            ui.label(format!("yaw: {:.2}", camera.get_yaw()));
                            ui.label(format!("pitch: {:.2}", camera.get_pitch()));
                            ui.label(format!("zoom: {:.2}", camera.get_zoom()));
                            ui.label(format!("near_plane: {:.2}", camera.get_near_plane()));
                            ui.label(format!("far_plane: {:.2}", camera.get_far_plane()));

                            ui.separator();

                            ui.label(format!(
                                "position: {}",
                                vec_to_string(&camera.get_position().normalize())
                            ));
                            ui.label(format!(
                                "front: {}",
                                vec_to_string(&camera.get_front().normalize())
                            ));
                        });
                    });

                let _output = egui.end_frame(glm::vec2(window_width as _, window_height as _));

                // set opengl viewport
                unsafe {
                    gl::Viewport(
                        viewport.0.try_into().unwrap(),
                        viewport.1.try_into().unwrap(),
                        viewport.2.try_into().unwrap(),
                        viewport.3.try_into().unwrap(),
                    );
                }
            }
            // GUI ends
        }

        // Swap front and back buffers
        window.swap_buffers();
    }

    // wait for all child threads to join
    ray_trace_thread_sender
        .send(RayTraceMessage::KillThread)
        .unwrap();
    ray_trace_main_thread_handle.join().unwrap();
}

fn vec_to_string<T: Scalar + std::fmt::Display, const R: usize>(vec: &glm::TVec<T, R>) -> String {
    let mut res = "[".to_string();
    for i in 0..R {
        if i != R - 1 {
            res = format!("{}{:.2}, ", res, vec[i]);
        } else {
            res = format!("{}{:.2}]", res, vec[i]);
        }
    }
    res
}

#[allow(clippy::too_many_arguments)]
fn handle_window_event(
    event: &glfw::WindowEvent,
    window: &mut glfw::Window,
    camera: &mut RasterizeCamera,
    path_trace_camera: &PathTraceCamera,
    should_cast_scene_ray: &mut bool,
    use_top_panel: &mut bool,
    use_bottom_panel: &mut bool,
    use_left_panel: &mut bool,
    use_right_panel: &mut bool,
    last_cursor: &mut (f64, f64),
) {
    let cursor = window.get_cursor_pos();
    match event {
        glfw::WindowEvent::Key(Key::Up, _, Action::Press, _) => {
            *use_top_panel = !*use_top_panel;
        }
        glfw::WindowEvent::Key(Key::Down, _, Action::Press, _) => {
            *use_bottom_panel = !*use_bottom_panel;
        }
        glfw::WindowEvent::Key(Key::Left, _, Action::Press, _) => {
            *use_left_panel = !*use_left_panel;
        }
        glfw::WindowEvent::Key(Key::Right, _, Action::Press, _) => {
            *use_right_panel = !*use_right_panel;
        }
        glfw::WindowEvent::Key(
            Key::Num1 | Key::Kp1,
            _,
            Action::Press,
            glfw::Modifiers::Control,
        ) => {
            *camera = RasterizeCamera::new(
                glm::vec3(0.0, 0.0, -camera.get_position().norm()),
                *camera.get_world_up(),
                90.0,
                0.0,
                camera.get_zoom(),
            )
        }
        glfw::WindowEvent::Key(Key::Num1 | Key::Kp1, _, Action::Press, _) => {
            *camera = RasterizeCamera::new(
                glm::vec3(0.0, 0.0, camera.get_position().norm()),
                *camera.get_world_up(),
                -90.0,
                0.0,
                camera.get_zoom(),
            )
        }
        glfw::WindowEvent::Key(
            Key::Num3 | Key::Kp3,
            _,
            Action::Press,
            glfw::Modifiers::Control,
        ) => {
            *camera = RasterizeCamera::new(
                glm::vec3(-camera.get_position().norm(), 0.0, 0.0),
                *camera.get_world_up(),
                0.0,
                0.0,
                camera.get_zoom(),
            )
        }
        glfw::WindowEvent::Key(Key::Num3 | Key::Kp3, _, Action::Press, _) => {
            *camera = RasterizeCamera::new(
                glm::vec3(camera.get_position().norm(), 0.0, 0.0),
                *camera.get_world_up(),
                180.0,
                0.0,
                camera.get_zoom(),
            )
        }
        glfw::WindowEvent::Key(
            Key::Num7 | Key::Kp7,
            _,
            Action::Press,
            glfw::Modifiers::Control,
        ) => {
            *camera = RasterizeCamera::new(
                glm::vec3(0.0, -camera.get_position().norm(), 0.0),
                *camera.get_world_up(),
                -90.0,
                90.0,
                camera.get_zoom(),
            )
        }
        glfw::WindowEvent::Key(Key::Num7 | Key::Kp7, _, Action::Press, _) => {
            *camera = RasterizeCamera::new(
                glm::vec3(0.0, camera.get_position().norm(), 0.0),
                *camera.get_world_up(),
                -90.0,
                -90.0,
                camera.get_zoom(),
            )
        }
        glfw::WindowEvent::Key(Key::Num0 | Key::Kp0, _, Action::Press, _) => {
            let fov = path_trace_camera
                .get_fov_hor()
                .max(path_trace_camera.get_fov_ver());
            *camera = RasterizeCamera::new(
                *path_trace_camera.get_origin(),
                *path_trace_camera.get_vertical(),
                -90.0,
                0.0,
                fov.to_degrees(),
            );
        }
        glfw::WindowEvent::Key(Key::C, _, Action::Press, glfw::Modifiers::Shift) => {
            let angle = camera.get_front().xz().angle(&-camera.get_position().xz());
            let distance_to_move = camera.get_position().xz().norm() * angle.sin();
            let move_vector = glm::vec3(camera.get_right()[0], 0.0, camera.get_right()[2])
                .normalize()
                * distance_to_move;
            let move_vector = if camera.get_right().dot(&camera.get_position()) > 0.0 {
                -move_vector
            } else {
                move_vector
            };
            *camera = RasterizeCamera::new(
                camera.get_position() + move_vector,
                *camera.get_world_up(),
                camera.get_yaw(),
                camera.get_pitch(),
                camera.get_zoom(),
            );
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
        && window.get_key(glfw::Key::LeftAlt) == glfw::Action::Press
    {
        *should_cast_scene_ray = true;
    }

    *last_cursor = cursor;
}
