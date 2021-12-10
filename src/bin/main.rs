use glm::Scalar;
use ipc_channel::ipc;
use rfd::FileDialog;
use rt::camera::{Camera, CameraDrawData};
use rt::image::Image;
use rt::inputs::InputArguments;
use rt::path_trace::environment::Environment;
use rt::path_trace::intersectable::Intersectable;
use rt::path_trace::medium::Mediums;
use rt::path_trace::ray::Ray;
use rt::path_trace::shader_list::{ShaderID, ShaderList};
use rt::path_trace::spectrum::{DSpectrum, SpectrumDrawData, Wavelengths};
use rt::path_trace::texture_list::TextureList;
use rt::path_trace::traversal_info::{TraversalInfo, TraversalInfoDrawData};
use rt::path_trace::viewport_renderer::{ViewportRenderer, ViewportRendererDrawData};
use rt::path_trace::{self, RayTraceMessage, RayTraceParams};
use rt::progress::Progress;
use rt::rasterize::gpu_utils::{self, draw_plane_with_image};
use rt::rasterize::texture::TextureRGBAFloat;
use rt::scene::{Scene, SceneDrawData};
use rt::ui::DrawUI;
use rt::viewport::Viewport;
use rt::{file, glm, icons, ui, util, UiData};

use std::cell::RefCell;
use std::convert::TryInto;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{self, AtomicBool};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

use egui_glfw::{
    egui::{self, FontDefinitions, FontFamily, TextStyle},
    EguiBackend,
};
use glfw::{Action, Context, Key};
use rand::seq::IteratorRandom;

use rt::fps::FPS;
use rt::rasterize::drawable::Drawable;
use rt::rasterize::gpu_immediate::GPUImmediate;
use rt::rasterize::infinite_grid::{InfiniteGrid, InfiniteGridDrawData};
use rt::rasterize::{shader, Rasterize};

fn print_active_feature_list() {
    print!("active_features: ");
    #[cfg(feature = "mesh_no_bvh")]
    print!("mesh_no_bvh, ");
    #[cfg(feature = "scene_no_bvh")]
    print!("scene_no_bvh, ");
    #[cfg(feature = "use_embree")]
    print!("use_embree, ");
    println!();
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

    print_active_feature_list();

    let arguments = InputArguments::read_cli();

    rayon::ThreadPoolBuilder::new()
        .num_threads(arguments.get_num_threads().unwrap_or(0))
        .build_global()
        .unwrap();

    let (ray_trace_params, scene, shader_list, texture_list, environment) =
        arguments.generate_render_info();

    let run_headless = arguments.get_run_headless();
    let path_trace_progress = Arc::new(RwLock::new(Progress::new()));
    let file_open_path = arguments.get_rt_file().cloned();
    let path_trace_camera = ray_trace_params.get_camera().clone();
    let rendered_image = ray_trace_params.get_rendered_image();

    // Spawn the main ray tracing thread
    let (ray_trace_thread_sender, ray_trace_thread_receiver) = mpsc::channel();
    let ray_trace_main_thread_handle = {
        let scene = scene.clone();
        let shader_list = shader_list.clone();
        let texture_list = texture_list.clone();
        let environment = environment.clone();
        let path_trace_progress = path_trace_progress.clone();
        thread::spawn(move || {
            path_trace::ray_trace_main(
                scene,
                shader_list,
                texture_list,
                environment,
                path_trace_progress,
                ray_trace_thread_receiver,
            );
        })
    };

    if run_headless {
        main_headless(
            ray_trace_main_thread_handle,
            ray_trace_thread_sender,
            path_trace_camera,
            rendered_image,
            path_trace_progress,
            arguments,
            sigint_triggered,
        );
    } else {
        main_gui(
            scene,
            shader_list,
            texture_list,
            Arc::new(RwLock::new(path_trace_camera)),
            environment,
            rendered_image,
            path_trace_progress,
            ray_trace_main_thread_handle,
            ray_trace_thread_sender,
            arguments,
            file_open_path,
            sigint_triggered,
        );
    }
}

fn main_headless(
    ray_trace_main_thread_handle: thread::JoinHandle<()>,
    ray_trace_thread_sender: mpsc::Sender<RayTraceMessage>,
    path_trace_camera: Camera,
    rendered_image: Arc<RwLock<Image>>,
    path_trace_progress: Arc<RwLock<Progress>>,
    arguments: InputArguments,
    sigint_triggered: Arc<AtomicBool>,
) {
    let image_width = arguments
        .get_image_width()
        .unwrap_or_else(rt::default_image_width);
    let image_height = arguments
        .get_image_height()
        .unwrap_or_else(rt::default_image_height);
    let trace_max_depth = arguments
        .get_trace_max_depth()
        .unwrap_or_else(rt::default_trace_max_depth);
    let samples_per_pixel = arguments
        .get_samples()
        .unwrap_or_else(rt::default_samples_per_pixel);

    ray_trace_thread_sender
        .send(RayTraceMessage::StartRender(RayTraceParams::new(
            image_width,
            image_height,
            trace_max_depth,
            samples_per_pixel,
            path_trace_camera,
            rendered_image.clone(),
        )))
        .unwrap();

    // setup progress sender if required and must send the total
    // number of samples first followed by the number of samples
    // completed updated as often as possible
    let path_trace_progress_sender = arguments
        .get_path_trace_progress_server_name()
        .map(|server_name| ipc::IpcSender::connect(server_name.to_string()).unwrap());

    // total number of samples must be kept consistent with ray trace
    // thread to ensure the progress bar shows things accurately
    let total_number_of_samples: u64 = (samples_per_pixel * image_width * image_height)
        .try_into()
        .unwrap();

    if let Some(sender) = &path_trace_progress_sender {
        sender.send(total_number_of_samples).unwrap();
    }

    let mut pb = pbr::ProgressBar::new(total_number_of_samples);

    pb.message("Tracing Scene: ");

    loop {
        if sigint_triggered.load(atomic::Ordering::SeqCst) {
            println!("waiting to finish current sample");
            ray_trace_thread_sender
                .send(RayTraceMessage::FinishSampleAndStopRender)
                .unwrap();
            break;
        }

        let progress = path_trace_progress.read().unwrap().get_progress();
        if (progress - 1.0).abs() < f64::EPSILON {
            pb.finish();
            break;
        }
        let progress = (progress * total_number_of_samples as f64) as u64;
        pb.set(progress);

        if let Some(sender) = &path_trace_progress_sender {
            sender.send(progress).unwrap();
        }
    }

    ray_trace_thread_sender
        .send(RayTraceMessage::FinishAndKillThread)
        .unwrap();

    ray_trace_main_thread_handle.join().unwrap();

    println!(
        "Finished in {}",
        util::duration_to_string(path_trace_progress.read().unwrap().get_elapsed_duration())
    );

    rt::save_image(
        &rendered_image.read().unwrap(),
        true,
        arguments.get_output_file().unwrap(),
    );
    println!(
        "saved rendered image to: {}",
        arguments.get_output_file().unwrap().to_str().unwrap()
    );
}

#[allow(clippy::too_many_arguments)]
fn main_gui(
    scene: Arc<RwLock<Scene>>,
    shader_list: Arc<RwLock<ShaderList>>,
    texture_list: Arc<RwLock<TextureList>>,
    path_trace_camera: Arc<RwLock<Camera>>,
    environment: Arc<RwLock<Environment>>,
    rendered_image: Arc<RwLock<Image>>,
    path_trace_progress: Arc<RwLock<Progress>>,
    ray_trace_main_thread_handle: thread::JoinHandle<()>,
    ray_trace_thread_sender: mpsc::Sender<RayTraceMessage>,
    arguments: InputArguments,
    mut file_open_path: Option<PathBuf>,
    sigint_triggered: Arc<AtomicBool>,
) {
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
        gl::Enable(gl::FRAMEBUFFER_SRGB);
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

    let mut camera = Camera::new(
        glm::vec3(0.0, 0.0, 3.0),
        glm::vec3(0.0, 1.0, 0.0),
        -90.0,
        0.0,
        45.0,
        None,
    );

    let imm = Rc::new(RefCell::new(GPUImmediate::new()));

    shader::builtins::display_uniform_and_attribute_info();

    let mut window_last_cursor = window.get_cursor_pos();

    let mut fps = FPS::default();

    let infinite_grid = InfiniteGrid::default();

    let rendered_texture = Rc::new(RefCell::new(TextureRGBAFloat::from_image(
        &rendered_image.read().unwrap(),
    )));
    let environment_texture = Rc::new(RefCell::new(TextureRGBAFloat::from_image(
        environment.read().unwrap().get_hdr(),
    )));

    let mut key_mods = glfw::Modifiers::empty();
    let mut use_top_panel = true;
    let mut use_bottom_panel = false;
    let mut use_left_panel = true;
    let mut use_right_panel = true;

    let mut viewport_rendered_shading: Option<ViewportRenderer> = None;
    let mut restart_viewport_rendered_shading = false;
    let mut open_rendered_image_window = false;
    let mut use_environment_map_as_background = false;
    let mut background_color = util::srgb_to_linear(&glm::vec4(0.051, 0.051, 0.051, 1.0));
    let mut infinite_grid_color = util::srgb_to_linear(&glm::vec4(0.15, 0.15, 0.15, 1.0));
    let mut should_cast_scene_ray = false;
    let mut try_select_object = false;
    let mut image_width = arguments
        .get_image_width()
        .unwrap_or_else(rt::default_image_width);
    let mut image_height = arguments
        .get_image_height()
        .unwrap_or_else(rt::default_image_height);
    let mut trace_max_depth = arguments
        .get_trace_max_depth()
        .unwrap_or_else(rt::default_trace_max_depth);
    let mut samples_per_pixel = arguments
        .get_samples()
        .unwrap_or_else(rt::default_samples_per_pixel);
    let mut ray_traversal_info: Vec<TraversalInfo> = Vec::new();
    let mut ray_to_shoot = (3, 3);
    let mut ray_pixel_start = (
        image_width / ray_to_shoot.0 / 2,
        image_height / ray_to_shoot.1 / 2,
    );
    let mut show_ray_traversal_info = true;
    let mut draw_normal_at_hit_points = true;
    let mut normals_size = 0.4;
    let mut normals_color = util::srgb_to_linear(&glm::vec4(1.0, 1.0, 1.0, 1.0));
    let mut camera_image_alpha_value = 0.0;
    let mut camera_use_depth_for_image = true;
    let mut selected_shader: Option<ShaderID> = None;
    let mut end_ray_depth: usize = trace_max_depth;
    let mut start_ray_depth: usize = 1;

    let mut spectrum_show = false;
    let mut spectrum_srgb_color = glm::vec3(0.0, 0.0, 0.0);

    let mut previous_frame_scene_viewport = None;

    while !window.should_close() {
        glfw.poll_events();

        glfw::flush_messages(&events).for_each(|(_, event)| {
            egui.handle_event(&event, &window);

            handle_window_event(
                &event,
                &mut window,
                &mut key_mods,
                &mut camera,
                &path_trace_camera.read().unwrap(),
                &mut should_cast_scene_ray,
                &mut try_select_object,
                &mut use_top_panel,
                &mut use_bottom_panel,
                &mut use_left_panel,
                &mut use_right_panel,
                &mut restart_viewport_rendered_shading,
                &mut window_last_cursor,
            );
        });

        if sigint_triggered.load(atomic::Ordering::SeqCst) {
            break;
        }

        // set window title
        if let Some(path) = file_open_path.as_ref() {
            window.set_title(&format!(
                "RT Rust ({})",
                path.canonicalize().unwrap().to_str().unwrap()
            ));
        } else {
            window.set_title("RT Rust");
        }

        rendered_texture
            .borrow_mut()
            .update_from_image(&rendered_image.read().unwrap());

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

        let window_viewport = {
            let (window_width, window_height) = window.get_size();
            Viewport::new(
                glm::vec2(
                    window_width.try_into().unwrap(),
                    window_height.try_into().unwrap(),
                ),
                glm::zero(),
            )
        };
        let framebuffer_viewport = {
            let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
            Viewport::new(
                glm::vec2(
                    framebuffer_width.try_into().unwrap(),
                    framebuffer_height.try_into().unwrap(),
                ),
                glm::zero(),
            )
        };
        let scene_viewport;

        // GUI starts
        {
            egui.begin_frame(&window, &mut glfw);

            let ui_data = UiData::new(
                scene.clone(),
                shader_list.clone(),
                texture_list.clone(),
                path_trace_camera.clone(),
                environment.clone(),
            );

            // Draw top, right, bottom and left panels, the order
            // matters since it goes from outermost to innermost.

            let top_panel_response = if use_top_panel {
                let response = egui::TopBottomPanel::top("Top Panel")
                    .resizable(true)
                    .show(egui.get_egui_ctx(), |ui| {
                        egui::menu::bar(ui, |ui| {
                            egui::menu::menu(ui, "File", |ui| {
                                if ui.button("Open").clicked() {
                                    if let Some(path) = FileDialog::new()
                                        .add_filter("RT", &["rt"])
                                        .add_filter("Any", &["*"])
                                        .set_directory(".")
                                        .pick_file()
                                    {
                                        file::load_rt_file(
                                            &path,
                                            scene.clone(),
                                            shader_list.clone(),
                                            path_trace_camera.clone(),
                                            environment.clone(),
                                        );

                                        file_open_path = Some(path);
                                    }
                                }

                                if ui.button("Revert").clicked() {
                                    if let Some(path) = file_open_path.as_ref() {
                                        file::load_rt_file(
                                            path,
                                            scene.clone(),
                                            shader_list.clone(),
                                            path_trace_camera.clone(),
                                            environment.clone(),
                                        );
                                    }
                                }

                                ui.separator();

                                let save_as = || {
                                    if let Some(path) = FileDialog::new()
                                        .add_filter("RT", &["rt"])
                                        .add_filter("Any", &["*"])
                                        .set_directory(".")
                                        .set_file_name("untitled.rt")
                                        .save_file()
                                    {
                                        file::save_rt_file(
                                            path,
                                            scene.clone(),
                                            shader_list.clone(),
                                            path_trace_camera.clone(),
                                            environment.clone(),
                                        );
                                    }
                                };

                                if ui.button("Save").clicked() {
                                    if let Some(path) = file_open_path.as_ref() {
                                        file::save_rt_file(
                                            path,
                                            scene.clone(),
                                            shader_list.clone(),
                                            path_trace_camera.clone(),
                                            environment.clone(),
                                        );
                                    } else {
                                        save_as();
                                    }
                                }

                                if ui.button("Save As").clicked() {
                                    save_as();
                                }

                                ui.separator();

                                if ui.button("Import OBJ").clicked() {
                                    if let Some(path) = FileDialog::new()
                                        .add_filter("OBJ", &["obj"])
                                        .add_filter("Any", &["*"])
                                        .set_directory(".")
                                        .pick_file()
                                    {
                                        rt::load_obj_file(path).drain(0..).for_each(|object| {
                                            scene.write().unwrap().add_object(Box::new(object));
                                        });
                                        // update scene bvh
                                        {
                                            let mut scene = scene.write().unwrap();
                                            scene.apply_model_matrices();

                                            scene.build_bvh(0.01);

                                            scene.unapply_model_matrices();
                                        }
                                    }
                                }

                                ui.separator();

                                if ui.button("Load Texture").clicked() {
                                    texture_list
                                        .write()
                                        .unwrap()
                                        .load_texture_with_file_dialog();
                                }

                                if ui.button("Load Environment Image").clicked() {
                                    environment.write().unwrap().load_hdr_file_dialog();
                                }
                            });
                        });
                    })
                    .response;
                Some(response)
            } else {
                None
            };

            let right_panel_response = if use_right_panel {
                let response = egui::SidePanel::right("Right Side Panel")
                    .min_width(0.1 * window_viewport.get_width() as f32)
                    .resizable(true)
                    .show(egui.get_egui_ctx(), |ui| {
                        egui::ScrollArea::auto_sized().show(ui, |ui| {
                            environment.read().unwrap().draw_ui(ui, &ui_data);
                            if let Ok(mut environment) = environment.try_write() {
                                environment.draw_ui_mut(ui, &ui_data);
                            } else {
                                ui.label("Environment currently in use, cannot edit environment");
                            }
                            environment_texture
                                .borrow_mut()
                                .update_from_image(environment.read().unwrap().get_hdr());
                            ui.collapsing("Environment Image", |ui| {
                                let environment_texture_ui_width =
                                    250.0_f32.min(0.3 * window_viewport.get_width() as f32);
                                let mut environment_texture = environment_texture.borrow_mut();
                                ui.image(
                                    egui::TextureId::User(environment_texture.get_gl_tex().into()),
                                    egui::vec2(
                                        environment_texture_ui_width,
                                        environment_texture_ui_width
                                            * environment_texture.get_height() as f32
                                            / environment_texture.get_width() as f32,
                                    ),
                                );
                            });

                            ui.separator();

                            shader_list.read().unwrap().draw_ui(ui, &ui_data);
                            if let Ok(mut shader_list) = shader_list.try_write() {
                                shader_list.draw_ui_mut(ui, &ui_data);
                                selected_shader = *shader_list.get_selected_shader();
                            } else {
                                ui.label("Shaders are currently in use, cannot edit the shaders");
                            }

                            ui.separator();

                            texture_list.read().unwrap().draw_ui(ui, &ui_data);
                            if let Ok(mut texture_list) = texture_list.try_write() {
                                texture_list.draw_ui_mut(ui, &ui_data);
                            } else {
                                ui.label("Textures are currently in use, cannot edit the textures");
                            }

                            ui.separator();

                            scene.read().unwrap().draw_ui(ui, &ui_data);
                            if let Ok(mut scene) = scene.try_write() {
                                scene.draw_ui_mut(ui, &ui_data);
                            } else {
                                ui.label("Scene is currently in use, cannot edit the scene");
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
                    .show(egui.get_egui_ctx(), |ui| {
                        ui.label(format!("mods: {}", glfw_modifier_to_string(key_mods)));
                    })
                    .response;
                Some(response)
            } else {
                None
            };

            let left_panel_response = if use_left_panel {
                let response = egui::SidePanel::left("Left Side Panel")
                    .min_width(0.1 * window_viewport.get_width() as f32)
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

                            ui.checkbox(
                                &mut use_environment_map_as_background,
                                "Use Environment Map as Background",
                            );

                            ui::color_edit_button_dvec4(
                                ui,
                                "Background Color",
                                &mut background_color,
                            );

                            ui::color_edit_button_dvec4(
                                ui,
                                "Infinite Grid Color",
                                &mut infinite_grid_color,
                            );

                            // For testing various things that are not
                            // directly connected to the path tracer
                            ui.collapsing("Testing Parameters", |ui| {
                                ui.collapsing("Spectrum Test", |ui| {
                                    ui.checkbox(&mut spectrum_show, "Show Spectrum");
                                    ui::color_edit_button_dvec3(
                                        ui,
                                        "Spectrum sRGB Color",
                                        &mut spectrum_srgb_color,
                                    );
                                });
                            });

                            ui.separator();

                            ui.checkbox(
                                &mut open_rendered_image_window,
                                "Open Rendered Image Window",
                            );

                            ui.add(
                                egui::Slider::new(&mut camera_image_alpha_value, 0.0..=1.0)
                                    .clamp_to_range(true)
                                    .text("Camera Image Alpha"),
                            );

                            ui.checkbox(&mut camera_use_depth_for_image, "Use Depth for Image");

                            ui.collapsing("Camera", |ui| {
                                let camera_sensor_width = {
                                    let mut camera_sensor_width = path_trace_camera
                                        .read()
                                        .unwrap()
                                        .get_sensor()
                                        .unwrap()
                                        .get_width();
                                    ui.add(
                                        egui::Slider::new(&mut camera_sensor_width, 0.0..=36.0)
                                            .text("Camera Sensor Width"),
                                    );
                                    camera_sensor_width
                                };

                                let camera_focal_length = {
                                    let mut camera_focal_length = path_trace_camera
                                        .read()
                                        .unwrap()
                                        .get_focal_length()
                                        .unwrap();
                                    ui.add(
                                        egui::Slider::new(&mut camera_focal_length, 0.0..=100.0)
                                            .text("Camera Focal Length"),
                                    );
                                    camera_focal_length
                                };

                                let camera_position = {
                                    let mut camera_position =
                                        path_trace_camera.read().unwrap().get_position();
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

                                ui.label("Camera Rotation");
                                let camera_yaw = {
                                    let mut camera_yaw =
                                        path_trace_camera.read().unwrap().get_yaw();
                                    ui.add(
                                        egui::Slider::new(&mut camera_yaw, 0.0..=360.0).text("yaw"),
                                    );
                                    camera_yaw
                                };
                                let camera_pitch = {
                                    let mut camera_pitch =
                                        path_trace_camera.read().unwrap().get_pitch();
                                    ui.add(
                                        egui::Slider::new(&mut camera_pitch, 0.0..=360.0)
                                            .text("pitch"),
                                    );
                                    camera_pitch
                                };

                                if let Ok(mut path_trace_camera) = path_trace_camera.try_write() {
                                    let sensor =
                                        path_trace_camera.get_sensor_mut().as_mut().unwrap();
                                    sensor.change_width(camera_sensor_width);
                                    sensor.change_aspect_ratio(
                                        image_width as f64 / image_height as f64,
                                    );
                                    path_trace_camera.set_focal_length(camera_focal_length);
                                    path_trace_camera.set_position(camera_position);
                                    path_trace_camera.set_yaw_and_pitch(camera_yaw, camera_pitch);
                                }
                            });

                            // path trace camera depends on image
                            // width and height which can be updated
                            // when camera header is
                            // closed. `ui.collapsing()` does not
                            // evaluate the function when it is closed
                            // thus the camera is not updated when the
                            // image width and height is modified and
                            // must be done so separately here
                            if let Ok(mut path_trace_camera) = path_trace_camera.try_write() {
                                let sensor = path_trace_camera.get_sensor_mut().as_mut().unwrap();
                                sensor
                                    .change_aspect_ratio(image_width as f64 / image_height as f64);
                            }

                            ui.separator();

                            ui.add(
                                egui::Slider::new(&mut image_width, 1..=1000).text("Image Width"),
                            );
                            if image_width == 0 {
                                image_width = 1;
                            }
                            ui.add(
                                egui::Slider::new(&mut image_height, 1..=1000).text("Image Height"),
                            );
                            if image_height == 0 {
                                image_height = 1;
                            }
                            let trace_max_depth_response = ui.add(
                                egui::Slider::new(&mut trace_max_depth, 1..=10)
                                    .text("Trace Max Depth"),
                            );
                            if trace_max_depth_response.changed() {
                                restart_viewport_rendered_shading = true;
                            }
                            let samples_per_pixel_response = ui.add(
                                egui::Slider::new(&mut samples_per_pixel, 1..=10)
                                    .text("Samples Per Pixel"),
                            );
                            if samples_per_pixel_response.changed() {
                                if samples_per_pixel == 0 {
                                    samples_per_pixel = 1;
                                }
                                restart_viewport_rendered_shading = true;
                            }

                            ui.horizontal(|ui| {
                                if ui.button("Ray Trace Scene").clicked() {
                                    ray_trace_thread_sender
                                        .send(RayTraceMessage::StartRender(RayTraceParams::new(
                                            image_width,
                                            image_height,
                                            trace_max_depth,
                                            samples_per_pixel,
                                            path_trace_camera.read().unwrap().clone(),
                                            rendered_image.clone(),
                                        )))
                                        .unwrap();
                                }

                                if ui.button("Stop Render").clicked() {
                                    ray_trace_thread_sender
                                        .send(RayTraceMessage::FinishSampleAndStopRender)
                                        .unwrap();
                                }
                            });

                            if ui.button("Save Ray Traced Image").clicked() {
                                if let Some(path) = FileDialog::new()
                                    .add_filter("image", &["image"])
                                    .add_filter("png", &["png"])
                                    .add_filter("jpg", &["jpg", "jpeg"])
                                    .add_filter("tiff", &["tiff"])
                                    .add_filter("Any", &["*"])
                                    .set_directory(".")
                                    .save_file()
                                {
                                    rt::save_image(&rendered_image.read().unwrap(), true, path);
                                }
                            }

                            ui.separator();

                            ui.collapsing("Ray", |ui| {
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

                                    scene.write().unwrap().rebuild_bvh_if_needed(0.01);

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

                                            let ray = path_trace_camera
                                                .get_ray(&glm::vec2(u, v))
                                                .unwrap();

                                            let environment: &Environment =
                                                &environment.read().unwrap();
                                            let (_color, traversal_info) = path_trace::trace_ray(
                                                &ray,
                                                &path_trace_camera,
                                                &scene.read().unwrap(),
                                                trace_max_depth,
                                                &shader_list.read().unwrap(),
                                                &texture_list.read().unwrap(),
                                                &environment.into(),
                                                &Wavelengths::complete(),
                                                &mut Mediums::with_air(),
                                            );
                                            ray_traversal_info.push(traversal_info);
                                        }
                                    }

                                    scene.write().unwrap().unapply_model_matrices();
                                }
                            });

                            if ui.button("Random assign shaders to objects").clicked() {
                                let shader_list = shader_list.read().unwrap();
                                let mut rng = rand::thread_rng();
                                scene.write().unwrap().get_objects_mut().for_each(|object| {
                                    let shader =
                                        shader_list.get_shaders().choose(&mut rng).unwrap();
                                    object.set_path_trace_shader_id(shader.get_shader_id());
                                });
                            }

                            if ui.button("Print object name shader name pairs ").clicked() {
                                let pairs =
                                    scene.read().unwrap().get_object_name_shader_name_pairs(
                                        &shader_list.read().unwrap(),
                                    );
                                println!("object name shader name pairs:");
                                pairs.iter().for_each(|(object_name, shader_name)| {
                                    if let Some(shader_name) = shader_name {
                                        println!("[\"{}\", \"{}\"],", object_name, shader_name);
                                    } else {
                                        println!("[\"{}\", None],", object_name);
                                    }
                                });
                            }
                        });
                    })
                    .response;
                Some(response)
            } else {
                None
            };

            scene_viewport = {
                let mut viewport_width = framebuffer_viewport.get_width();
                let mut viewport_height = framebuffer_viewport.get_height();

                let viewport_top_left_y = if let Some(top_panel_response) = top_panel_response {
                    viewport_height -= top_panel_response.rect.size().y as isize;
                    top_panel_response.rect.size().y as isize
                } else {
                    0
                };
                if let Some(bottom_panel_response) = bottom_panel_response {
                    viewport_height -= bottom_panel_response.rect.size().y as isize;
                }
                let viewport_top_left_x = if let Some(left_panel_response) = left_panel_response {
                    viewport_width -= left_panel_response.rect.size().x as isize;
                    left_panel_response.rect.size().x as isize
                } else {
                    0
                };
                if let Some(right_panel_response) = right_panel_response {
                    viewport_width -= right_panel_response.rect.size().x as isize;
                }

                Viewport::new(
                    glm::vec2(viewport_width.max(1), viewport_height.max(1)),
                    glm::vec2(viewport_top_left_x, viewport_top_left_y),
                )
            };

            if let Some(previous_frame_scene_viewport) = previous_frame_scene_viewport {
                if previous_frame_scene_viewport != scene_viewport {
                    restart_viewport_rendered_shading = true;
                }
            }

            if restart_viewport_rendered_shading {
                if let Some(viewport_rendered_shading) = viewport_rendered_shading.as_ref() {
                    viewport_rendered_shading.restart_render(
                        scene_viewport.clone(),
                        trace_max_depth,
                        samples_per_pixel,
                        camera.clone(),
                    );
                }
                restart_viewport_rendered_shading = false;
            }

            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
                .show(egui.get_egui_ctx(), |ui| {
                    egui::TopBottomPanel::top("viewport top panel").show_inside(ui, |ui| {
                        ui.with_layout(ui.layout().with_cross_align(egui::Align::RIGHT), |ui| {
                            ui.horizontal(|ui| {
                                // ordering is from right to left
                                {
                                    if ui
                                        .add(egui::ImageButton::new(
                                            egui::TextureId::User(
                                                icons::get_viewport_rendered_shading_icon_id()
                                                    .into(),
                                            ),
                                            [ui.available_height(), ui.available_height()],
                                        ))
                                        .clicked()
                                    {
                                        viewport_rendered_shading = Some(ViewportRenderer::new(
                                            scene_viewport.clone(),
                                            trace_max_depth,
                                            samples_per_pixel,
                                            camera.clone(),
                                            path_trace_progress.clone(),
                                            ray_trace_thread_sender.clone(),
                                        ));
                                    }

                                    if ui
                                        .add(egui::ImageButton::new(
                                            egui::TextureId::User(
                                                icons::get_viewport_solid_shading_icon_id().into(),
                                            ),
                                            [ui.available_height(), ui.available_height()],
                                        ))
                                        .clicked()
                                    {
                                        viewport_rendered_shading = None;
                                    }
                                }

                                ui.with_layout(egui::Layout::left_to_right(), |_ui| {
                                    // Any UI elements that go on the
                                    // left side, left to right
                                    // ordering
                                });
                            });
                        });
                    });
                });

            egui::Window::new("Rendered Image Window")
                .open(&mut open_rendered_image_window)
                .collapsible(true)
                .resize(|r| {
                    r.resizable(true).max_size(egui::vec2(
                        scene_viewport.get_width() as f32,
                        scene_viewport.get_height() as f32,
                    ))
                })
                .scroll(true)
                .show(egui.get_egui_ctx(), |ui| {
                    let mut rendered_texture = rendered_texture.borrow_mut();
                    ui.image(
                        egui::TextureId::User(rendered_texture.get_gl_tex().into()),
                        egui::vec2(
                            rendered_texture.get_width() as f32,
                            rendered_texture.get_height() as f32,
                        ),
                    );
                });

            egui::Window::new("Camera Data")
                .open(&mut false)
                .collapsible(true)
                .scroll(true)
                .show(egui.get_egui_ctx(), |ui| {
                    // let camera = &camera;
                    let camera = &path_trace_camera.read().unwrap();
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
                    ui.label(format!("fov: {:.2}", camera.get_fov()));
                    ui.label(format!("near_plane: {:.2}", camera.get_near_plane()));
                    ui.label(format!("far_plane: {:.2}", camera.get_far_plane()));

                    if let Some(sensor) = camera.get_sensor() {
                        ui.label(format!("sensor width: {}", sensor.get_width()));
                        ui.label(format!("sensor height: {}", sensor.get_height()));
                        ui.label(format!(
                            "sensor aspect ratio: {}",
                            sensor.get_aspect_ratio()
                        ));
                    } else {
                        ui.label("sensor not available");
                    }

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
        }
        // GUI Ends

        let scene_last_cursor_pos = scene_viewport.calculate_location((
            &glm::vec2(window_last_cursor.0 as _, window_last_cursor.1 as _),
            &window_viewport,
        ));

        // set opengl viewport
        scene_viewport.set_opengl_viewport(&window_viewport);

        // Shader stuff
        shader::builtins::setup_shaders(
            &camera,
            scene_viewport.get_width().try_into().unwrap(),
            scene_viewport.get_height().try_into().unwrap(),
        );

        // Disable blending, render only opaque objects
        unsafe {
            gl::Disable(gl::BLEND);
        }

        // Sky Box
        if use_environment_map_as_background {
            let srgb_not_on;
            unsafe {
                gl::Disable(gl::DEPTH_TEST);
                srgb_not_on = gl::IsEnabled(gl::FRAMEBUFFER_SRGB) == gl::FALSE;
                gl::Enable(gl::FRAMEBUFFER_SRGB);
            }

            let environment_shader = shader::builtins::get_environment_shader().as_ref().unwrap();
            environment_shader.use_shader();
            environment_shader.set_int("environment_map\0", 30);
            let environment = environment.read().unwrap();
            environment_shader.set_mat4(
                "model\0",
                &glm::convert(environment.get_transform().get_matrix()),
            );
            environment_shader.set_float("strength\0", environment.get_strength() as _);
            environment_texture.borrow_mut().activate(30);

            gpu_utils::draw_screen_quad(&mut imm.borrow_mut(), environment_shader);

            unsafe {
                gl::Enable(gl::DEPTH_TEST);
                if srgb_not_on {
                    gl::Disable(gl::FRAMEBUFFER_SRGB);
                }
            }
        }

        // drawing the scene
        scene
            .read()
            .unwrap()
            .draw(&mut SceneDrawData::new(imm.clone(), shader_list.clone()))
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
            &glm::vec3(200.0, image_height as f64 / 1000.0, 0.0),
            &glm::vec3(image_width as f64 / 500.0, 2.0, image_height as f64 / 500.0),
            &glm::vec3(0.0, 0.0, -1.0),
            &mut rendered_texture.borrow_mut(),
            1.0,
            &mut imm.borrow_mut(),
        );

        // spectrum drawing test
        if spectrum_show {
            let spectrum = DSpectrum::from_srgb(&spectrum_srgb_color);
            let spectrum_r = DSpectrum::from_srgb(&glm::vec3(spectrum_srgb_color[0], 0.0, 0.0));
            let spectrum_g = DSpectrum::from_srgb(&glm::vec3(0.0, spectrum_srgb_color[1], 0.0));
            let spectrum_b = DSpectrum::from_srgb(&glm::vec3(0.0, 0.0, spectrum_srgb_color[2]));
            let position = glm::vec3(-1.0, 0.0, 0.0);
            let scale = glm::vec3(2.0, 1.0, 1.0);
            let normal = glm::vec3(0.0, 0.0, -1.0);
            let draw_spectrum = |spectrum: &DSpectrum| {
                spectrum
                    .draw(&mut SpectrumDrawData::new(
                        imm.clone(),
                        position,
                        scale,
                        normal,
                    ))
                    .unwrap();
            };

            draw_spectrum(&spectrum);
            draw_spectrum(&spectrum_r);
            draw_spectrum(&spectrum_g);
            draw_spectrum(&spectrum_b);
        }

        // handle casting ray into the scene
        if should_cast_scene_ray {
            let ray_direction = camera.get_raycast_direction(
                scene_last_cursor_pos[0] as f64,
                scene_last_cursor_pos[1] as f64,
                scene_viewport.get_width().try_into().unwrap(),
                scene_viewport.get_height().try_into().unwrap(),
            );

            scene.write().unwrap().apply_model_matrices();

            scene.write().unwrap().rebuild_bvh_if_needed(0.01);

            let path_trace_camera = path_trace_camera.read().unwrap();

            // trace ray into scene from the rasterizer camera
            // position to get the first hitpoint
            let environment: &Environment = &environment.read().unwrap();
            let (_color, traversal_info) = path_trace::trace_ray(
                &Ray::new(camera.get_position(), ray_direction),
                &path_trace_camera,
                &scene.read().unwrap(),
                1,
                &shader_list.read().unwrap(),
                &texture_list.read().unwrap(),
                &environment.into(),
                &Wavelengths::complete(),
                &mut Mediums::with_air(),
            );

            // generate the new ray from the path_trace_camera's
            // position towards the first hitpoint
            let ray_direction = if let Some(hit_point) = traversal_info.get_traversal()[0].get_co()
            {
                (hit_point - path_trace_camera.get_position()).normalize()
            } else {
                (traversal_info.get_traversal()[0].get_ray().at(1000.0)
                    - path_trace_camera.get_position())
                .normalize()
            };

            let (_color, traversal_info) = path_trace::trace_ray(
                &Ray::new(path_trace_camera.get_position(), ray_direction),
                &path_trace_camera,
                &scene.read().unwrap(),
                trace_max_depth,
                &shader_list.read().unwrap(),
                &texture_list.read().unwrap(),
                &environment.into(),
                &Wavelengths::complete(),
                &mut Mediums::with_air(),
            );

            scene.write().unwrap().unapply_model_matrices();

            ray_traversal_info.clear();
            ray_traversal_info.push(traversal_info);

            should_cast_scene_ray = false;
        }

        // select object
        if try_select_object {
            if let Ok(mut scene) = scene.try_write() {
                let ray_direction = camera.get_raycast_direction(
                    scene_last_cursor_pos[0] as f64,
                    scene_last_cursor_pos[1] as f64,
                    scene_viewport.get_width().try_into().unwrap(),
                    scene_viewport.get_height().try_into().unwrap(),
                );

                scene.apply_model_matrices();

                scene.rebuild_bvh_if_needed(0.01);

                scene.try_select_object(&Ray::new(camera.get_position(), ray_direction));

                scene.unapply_model_matrices();
            }
            try_select_object = false;
        }

        if window.get_mouse_button(glfw::MouseButtonLeft) == glfw::Action::Press {
            if let Some(shader_id) = selected_shader {
                let ray_direction = camera.get_raycast_direction(
                    scene_last_cursor_pos[0] as f64,
                    scene_last_cursor_pos[1] as f64,
                    scene_viewport.get_width().try_into().unwrap(),
                    scene_viewport.get_height().try_into().unwrap(),
                );

                let mut scene = scene.write().unwrap();
                scene.apply_model_matrices();
                scene.rebuild_bvh_if_needed(0.01);
                if let Some(hit_info) = scene.hit(
                    &Ray::new(camera.get_position(), ray_direction),
                    0.01,
                    1000.0,
                ) {
                    let object_id = hit_info.get_object_id().unwrap();
                    scene.get_objects_mut().for_each(|object| {
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

            if let Some(viewport_renderer) = viewport_rendered_shading.as_ref() {
                viewport_renderer
                    .draw(&mut ViewportRendererDrawData::new(imm.clone()))
                    .unwrap();
            }

            // drawing the camera
            path_trace_camera
                .read()
                .unwrap()
                .draw(&mut CameraDrawData::new(
                    imm.clone(),
                    Some(rendered_texture.clone()),
                    camera_image_alpha_value,
                    camera_use_depth_for_image,
                ))
                .unwrap();

            // drawing the infinite grid
            infinite_grid
                .draw(&mut InfiniteGridDrawData::new(
                    imm.clone(),
                    infinite_grid_color,
                ))
                .unwrap();

            // Draw GUI
            {
                // set the opengl viewport for the full frame buffer
                // for correct GUI element drawing
                framebuffer_viewport.set_opengl_viewport(&window_viewport);
                let _output = egui.end_frame(glm::vec2(
                    framebuffer_viewport.get_width() as _,
                    framebuffer_viewport.get_height() as _,
                ));
            }
        }

        previous_frame_scene_viewport = Some(scene_viewport);

        // Swap front and back buffers
        window.swap_buffers();
    }

    handle_opengl_cleanup(scene, texture_list);

    // turn off viewport rendered shading so cleanup happens
    // correctly, must be cleaned up before ray trace main thread is
    // killed
    drop(viewport_rendered_shading);

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
    key_mods: &mut glfw::Modifiers,
    camera: &mut Camera,
    path_trace_camera: &Camera,
    should_cast_scene_ray: &mut bool,
    try_select_object: &mut bool,
    use_top_panel: &mut bool,
    use_bottom_panel: &mut bool,
    use_left_panel: &mut bool,
    use_right_panel: &mut bool,
    restart_viewport_rendered_shading: &mut bool,
    window_last_cursor: &mut (f64, f64),
) {
    let window_cursor = window.get_cursor_pos();

    match event {
        glfw::WindowEvent::Key(_, _, Action::Press, mods) => *key_mods |= *mods,
        glfw::WindowEvent::Key(_, _, Action::Release, mods) => *key_mods &= !*mods,
        glfw::WindowEvent::CharModifiers(_, mods) => *key_mods |= *mods,
        glfw::WindowEvent::MouseButton(_, Action::Press, mods) => *key_mods |= *mods,
        glfw::WindowEvent::MouseButton(_, Action::Release, mods) => *key_mods &= !*mods,
        _ => {}
    }

    match event {
        glfw::WindowEvent::Key(Key::Up, _, Action::Press, glfw::Modifiers::Alt) => {
            *use_top_panel = !*use_top_panel;
        }
        glfw::WindowEvent::Key(Key::Down, _, Action::Press, glfw::Modifiers::Alt) => {
            *use_bottom_panel = !*use_bottom_panel;
        }
        glfw::WindowEvent::Key(Key::Left, _, Action::Press, glfw::Modifiers::Alt) => {
            *use_left_panel = !*use_left_panel;
        }
        glfw::WindowEvent::Key(Key::Right, _, Action::Press, glfw::Modifiers::Alt) => {
            *use_right_panel = !*use_right_panel;
        }
        glfw::WindowEvent::Key(Key::Num1 | Key::Kp1, _, Action::Press, modifier) => {
            if modifier.contains(glfw::Modifiers::Control | glfw::Modifiers::Alt) {
                *camera = Camera::new(
                    glm::vec3(0.0, 0.0, -camera.get_position().norm()),
                    *camera.get_world_up(),
                    90.0,
                    0.0,
                    camera.get_fov(),
                    camera.get_sensor_no_ref(),
                )
            } else if modifier.contains(glfw::Modifiers::Alt) {
                *camera = Camera::new(
                    glm::vec3(0.0, 0.0, camera.get_position().norm()),
                    *camera.get_world_up(),
                    -90.0,
                    0.0,
                    camera.get_fov(),
                    camera.get_sensor_no_ref(),
                )
            }
            *restart_viewport_rendered_shading = true;
        }
        glfw::WindowEvent::Key(Key::Num3 | Key::Kp3, _, Action::Press, modifier) => {
            if modifier.contains(glfw::Modifiers::Control | glfw::Modifiers::Alt) {
                *camera = Camera::new(
                    glm::vec3(-camera.get_position().norm(), 0.0, 0.0),
                    *camera.get_world_up(),
                    0.0,
                    0.0,
                    camera.get_fov(),
                    camera.get_sensor_no_ref(),
                )
            } else if modifier.contains(glfw::Modifiers::Alt) {
                *camera = Camera::new(
                    glm::vec3(camera.get_position().norm(), 0.0, 0.0),
                    *camera.get_world_up(),
                    180.0,
                    0.0,
                    camera.get_fov(),
                    camera.get_sensor_no_ref(),
                )
            }
            *restart_viewport_rendered_shading = true;
        }
        glfw::WindowEvent::Key(Key::Num7 | Key::Kp7, _, Action::Press, modifier) => {
            if modifier.contains(glfw::Modifiers::Control | glfw::Modifiers::Alt) {
                *camera = Camera::new(
                    glm::vec3(0.0, -camera.get_position().norm(), 0.0),
                    *camera.get_world_up(),
                    -90.0,
                    90.0,
                    camera.get_fov(),
                    camera.get_sensor_no_ref(),
                )
            } else if modifier.contains(glfw::Modifiers::Alt) {
                *camera = Camera::new(
                    glm::vec3(0.0, camera.get_position().norm(), 0.0),
                    *camera.get_world_up(),
                    -90.0,
                    -90.0,
                    camera.get_fov(),
                    camera.get_sensor_no_ref(),
                )
            }
            *restart_viewport_rendered_shading = true;
        }
        glfw::WindowEvent::Key(Key::Num0 | Key::Kp0, _, Action::Press, modifier) => {
            if modifier.contains(glfw::Modifiers::Alt) {
                *camera = path_trace_camera.clone();
            }
            *restart_viewport_rendered_shading = true;
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
            *camera = Camera::new(
                camera.get_position() + move_vector,
                *camera.get_world_up(),
                camera.get_yaw(),
                camera.get_pitch(),
                camera.get_fov(),
                camera.get_sensor_no_ref(),
            );
            *restart_viewport_rendered_shading = true;
        }

        glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
            gl::Viewport(0, 0, *width, *height);
            *restart_viewport_rendered_shading = true;
        },
        glfw::WindowEvent::Scroll(_, scroll_y) => {
            camera.zoom(*scroll_y);
            *restart_viewport_rendered_shading = true;
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
                window_last_cursor.0,
                window_last_cursor.1,
                window_cursor.0,
                window_cursor.1,
                1.0,
                window_width,
                window_height,
            );
        } else if window.get_key(glfw::Key::LeftControl) == glfw::Action::Press {
            camera.move_forward(window_last_cursor.1, window_cursor.1, window_height);
        } else {
            camera.rotate_wrt_camera_origin(
                window_last_cursor.0,
                window_last_cursor.1,
                window_cursor.0,
                window_cursor.1,
                0.1,
                false,
            );
        }
        *restart_viewport_rendered_shading = true;
    }

    if window.get_mouse_button(glfw::MouseButtonLeft) == glfw::Action::Press
        && window.get_key(glfw::Key::LeftAlt) == glfw::Action::Press
    {
        *should_cast_scene_ray = true;
    }

    if window.get_mouse_button(glfw::MouseButtonRight) == glfw::Action::Press {
        *try_select_object = true;
    }

    *window_last_cursor = window_cursor;
}

/// OpenGL commands need to be executed while a context is
/// active. Sometimes it is possible to loose the context prior to
/// cleaning up all the OpenGL resources. See
/// <https://www.khronos.org/opengl/wiki/Common_Mistakes#The_Object_Oriented_Language_Problem>
/// for more details.
///
/// In this case, any opengl related objects created before
/// [`main_gui()`] (in [`main()`]) must be cleaned up
fn handle_opengl_cleanup(scene: Arc<RwLock<Scene>>, texture_list: Arc<RwLock<TextureList>>) {
    scene.write().unwrap().cleanup_opengl();
    texture_list.write().unwrap().cleanup_opengl();
}

fn glfw_modifier_to_string(mods: glfw::Modifiers) -> String {
    let res = if mods.contains(glfw::Modifiers::Shift) {
        "Shift".to_string()
    } else {
        "".to_string()
    };
    let res = if mods.contains(glfw::Modifiers::Control) {
        if !res.is_empty() {
            format!("{} + Ctrl", res)
        } else {
            "Ctrl".to_string()
        }
    } else {
        res
    };
    let res = if mods.contains(glfw::Modifiers::Alt) {
        if !res.is_empty() {
            format!("{} + Alt", res)
        } else {
            "Alt".to_string()
        }
    } else {
        res
    };
    let res = if mods.contains(glfw::Modifiers::Super) {
        if !res.is_empty() {
            format!("{} + Super", res)
        } else {
            "Super".to_string()
        }
    } else {
        res
    };
    let res = if mods.contains(glfw::Modifiers::CapsLock) {
        if !res.is_empty() {
            format!("{} + Caps Lock", res)
        } else {
            "Caps Lock".to_string()
        }
    } else {
        res
    };
    let res = if mods.contains(glfw::Modifiers::NumLock) {
        if !res.is_empty() {
            format!("{} + Num Lock", res)
        } else {
            "Num Lock".to_string()
        }
    } else {
        res
    };
    res
}
