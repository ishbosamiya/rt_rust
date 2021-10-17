use rt::bvh::BVHTree;
use rt::glm;
use rt::image::{Image, PPM};
use rt::object::objects::Sphere as SphereObject;
use rt::object::ObjectDrawData;
use rt::path_trace;
use rt::path_trace::camera::Camera as PathTraceCamera;
use rt::rasterize::gpu_utils::draw_plane_with_image;
use rt::rasterize::texture::TextureRGBAFloat;
use rt::scene::Scene;
use rt::sphere::Sphere;

extern crate lazy_static;
use rayon::prelude::*;

fn ray_trace_scene(
    width: usize,
    height: usize,
    trace_max_depth: usize,
    samples_per_pixel: usize,
    scene: &Scene,
) -> Image {
    let mut image = Image::new(width, height);

    let viewport_height = 2.0;
    let aspect_ratio = width as f64 / height as f64;
    let focal_length = 1.0;
    let origin = glm::vec3(0.0, 0.0, 0.0);
    let camera = &PathTraceCamera::new(viewport_height, aspect_ratio, focal_length, origin);

    image
        .get_pixels_mut()
        .par_iter_mut()
        .enumerate()
        .for_each(|(j, row)| {
            row.par_iter_mut().enumerate().for_each(|(i, pixel)| {
                *pixel = glm::vec3(0.0, 0.0, 0.0);
                for _ in 0..samples_per_pixel {
                    let j = height - j - 1;

                    // use opengl coords, (0.0, 0.0) is center; (1.0, 1.0) is
                    // top right; (-1.0, -1.0) is bottom left
                    let u = (((i as f64 + rand::random::<f64>()) / (width - 1) as f64) - 0.5) * 2.0;
                    let v =
                        (((j as f64 + rand::random::<f64>()) / (height - 1) as f64) - 0.5) * 2.0;

                    let ray = camera.get_ray(u, v);

                    *pixel += path_trace::trace_ray(&ray, camera, scene, trace_max_depth);
                }
                *pixel /= samples_per_pixel as f64;
            });
        });

    image
}

use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

use egui::{FontDefinitions, FontFamily, TextStyle};
use egui_glfw::EguiBackend;
use glfw::{Action, Context, Key};

use rt::fps::FPS;
use rt::mesh;
use rt::mesh::{MeshDrawData, MeshUseShader};
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

    let mesh = mesh::builtins::get_monkey_subd_01();

    let mut camera = RasterizeCamera::new(
        glm::vec3(0.0, 0.0, 3.0),
        glm::vec3(0.0, 1.0, 0.0),
        -90.0,
        0.0,
        45.0,
    );

    let imm = Rc::new(RefCell::new(GPUImmediate::new()));

    shader::builtins::display_uniform_and_attribute_info();

    let directional_light_shader = shader::builtins::get_directional_light_shader()
        .as_ref()
        .unwrap();

    let mut last_cursor = window.get_cursor_pos();

    let mut fps = FPS::default();

    let mut draw_bvh = true;
    let mut bvh_draw_level = 0;
    let mut should_cast_ray = false;
    let mut bvh_color = glm::vec4(0.9, 0.5, 0.2, 1.0);
    let mut bvh_ray_color: glm::DVec4 = glm::vec4(0.2, 0.5, 0.9, 1.0);
    let mut bvh_ray_intersection = Vec::new();
    let mut image_width = 1000;
    let mut image_height = 1000;
    let mut trace_max_depth = 5;
    let mut samples_per_pixel = 5;
    let mut save_image_location = "test.ppm".to_string();

    let scene = {
        let mut scene = Scene::new();
        scene.add_object(Box::new(SphereObject::new(
            Sphere::new(glm::vec3(0.0, 0.0, -2.0), 0.45),
            glm::vec4(0.0, 0.0, 1.0, 1.0),
            glm::vec4(1.0, 0.0, 0.0, 1.0),
        )));
        scene.add_object(Box::new(SphereObject::new(
            Sphere::new(glm::vec3(0.0, 0.0, -2.0), 0.45),
            glm::vec4(0.0, 0.0, 1.0, 1.0),
            glm::vec4(1.0, 0.0, 0.0, 1.0),
        )));
        scene.add_object(Box::new(SphereObject::new(
            Sphere::new(glm::vec3(0.0, 1.0, -2.0), 0.45),
            glm::vec4(0.0, 0.0, 1.0, 1.0),
            glm::vec4(1.0, 0.0, 0.0, 1.0),
        )));
        scene.add_object(Box::new(SphereObject::new(
            Sphere::new(glm::vec3(0.0, -1.0, -2.0), 0.45),
            glm::vec4(0.0, 0.0, 1.0, 1.0),
            glm::vec4(1.0, 0.0, 0.0, 1.0),
        )));
        scene.add_object(Box::new(SphereObject::new(
            Sphere::new(glm::vec3(1.0, 0.0, -2.0), 0.45),
            glm::vec4(0.0, 0.0, 1.0, 1.0),
            glm::vec4(1.0, 0.0, 0.0, 1.0),
        )));
        scene.add_object(Box::new(SphereObject::new(
            Sphere::new(glm::vec3(-1.0, 0.0, -2.0), 0.45),
            glm::vec4(0.0, 0.0, 1.0, 1.0),
            glm::vec4(1.0, 0.0, 0.0, 1.0),
        )));
        scene
    };

    let infinite_grid = InfiniteGrid::default();

    let mut image = TextureRGBAFloat::new_empty(100, 100);

    while !window.should_close() {
        glfw.poll_events();

        glfw::flush_messages(&events).for_each(|(_, event)| {
            egui.handle_event(&event, &window);

            handle_window_event(
                &event,
                &mut window,
                &mut camera,
                &mut should_cast_ray,
                &mut last_cursor,
            );
        });

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
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

        scene.draw(&mut ObjectDrawData::new(imm.clone())).unwrap();

        directional_light_shader.use_shader();
        directional_light_shader.set_mat4("model\0", &glm::identity());
        mesh.draw(&mut MeshDrawData::new(
            imm.clone(),
            MeshUseShader::DirectionalLight,
            draw_bvh,
            bvh_draw_level,
            bvh_color,
        ))
        .unwrap();

        draw_plane_with_image(
            &glm::vec3(2.0, image_height as f64 / 1000.0, 0.0),
            &glm::vec3(image_width as f64 / 500.0, 2.0, image_height as f64 / 500.0),
            &glm::vec3(0.0, 0.0, 1.0),
            &mut image,
            &mut imm.borrow_mut(),
        );

        if should_cast_ray {
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

            should_cast_ray = false;
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

            infinite_grid
                .draw(&mut InfiniteGridDrawData::new(imm.clone()))
                .unwrap();

            // GUI starts
            {
                egui.begin_frame(&window, &mut glfw);
                egui::Window::new("Hello world!").show(egui.get_egui_ctx(), |ui| {
                    ui.label("Hello RT Rust!");
                    ui.label(format!("fps: {:.2}", fps.update_and_get(Some(60.0))));

                    ui.checkbox(&mut draw_bvh, "Draw BVH");
                    ui.add(egui::Slider::new(&mut bvh_draw_level, 0..=15).text("BVH Draw Level"));
                    color_edit_button_dvec4(ui, "BVH Color", &mut bvh_color);
                    color_edit_button_dvec4(ui, "BVH Ray Color", &mut bvh_ray_color);

                    if ui.button("Delete Rays").clicked() {
                        bvh_ray_intersection.clear();
                    }

                    ui.separator();

                    ui.add(egui::Slider::new(&mut image_width, 1..=1000).text("Image Width"));
                    ui.add(egui::Slider::new(&mut image_height, 1..=1000).text("Image Height"));
                    ui.add(egui::Slider::new(&mut trace_max_depth, 1..=10).text("Trace Max Depth"));
                    ui.add(
                        egui::Slider::new(&mut samples_per_pixel, 1..=10).text("Samples Per Pixel"),
                    );

                    if ui.button("Ray Trace Scene").clicked() {
                        image = TextureRGBAFloat::from_image(&ray_trace_scene(
                            image_width,
                            image_height,
                            trace_max_depth,
                            samples_per_pixel,
                            &scene,
                        ));
                    }

                    ui.horizontal(|ui| {
                        ui.label("Save Location");
                        ui.text_edit_singleline(&mut save_image_location);
                    });

                    if ui.button("Save Ray Traced Image").clicked() {
                        let image = Image::from_texture_rgba_float(&image);
                        PPM::new(&image)
                            .write_to_file(&save_image_location)
                            .unwrap();
                    }
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
    should_cast_ray: &mut bool,
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
        *should_cast_ray = true;
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
