use rt::bvh::BVHTree;
use rt::camera::Camera;
use rt::image::{Image, PPM};
use rt::math::Scalar;
use rt::scene::Scene;
use rt::sphere::Sphere;

use rt::trace_ray;

use nalgebra_glm as glm;
extern crate lazy_static;
use crossbeam::thread;
use lazy_static::lazy_static;

lazy_static! {
    static ref SCENE: Scene = {
        let mut scene = Scene::new();
        scene.add_object(Box::new(Sphere::new(glm::vec3(0.0, 0.0, -2.0), 1.5)));
        scene.add_object(Box::new(Sphere::new(glm::vec3(0.0, 1.0, -2.0), 1.5)));
        scene.add_object(Box::new(Sphere::new(glm::vec3(0.0, -1.0, -2.0), 1.5)));
        scene.add_object(Box::new(Sphere::new(glm::vec3(1.0, 0.0, -2.0), 1.5)));
        scene.add_object(Box::new(Sphere::new(glm::vec3(-1.0, 0.0, -2.0), 1.5)));
        scene
    };
}

fn _ray_trace_scene() {
    let width = 1000;
    let height = 1000;
    let mut image = Image::new(width, height);

    let viewport_height = 2.0;
    let aspect_ratio = width as f64 / height as f64;
    let focal_length = 1.0;
    let origin = glm::vec3(0.0, 0.0, 0.0);
    let camera = Camera::new(viewport_height, aspect_ratio, focal_length, origin);
    let camera = &camera;

    {
        let num_threads = 12;
        let mut slabs = image.get_slabs(num_threads);

        println!("slabs: {:?}", slabs);

        thread::scope(|s| {
            let mut handles = Vec::new();

            for slab in &mut slabs {
                let handle = s.spawn(move |_| {
                    let mut pixels = Vec::new();
                    for i in 0..slab.width {
                        let mut pixels_inner = Vec::new();
                        for j in 0..slab.height {
                            let j = j + slab.y_start;
                            let j = height - j;
                            let i = i + slab.x_start;

                            // use opengl coords, (0.0, 0.0) is center; (1.0, 1.0) is
                            // top right; (-1.0, -1.0) is bottom left
                            let u = ((i as Scalar / (width - 1) as Scalar) - 0.5) * 2.0;
                            let v = ((j as Scalar / (height - 1) as Scalar) - 0.5) * 2.0;

                            let ray = camera.get_ray(u, v);

                            let pixel = trace_ray(&ray, camera, &SCENE, 2000);
                            pixels_inner.push(pixel);
                        }
                        pixels.push(pixels_inner);
                    }

                    slab.set_pixels(pixels);
                });

                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        })
        .unwrap();

        for slab in slabs {
            for i in 0..slab.width {
                for j in 0..slab.height {
                    let pixel = slab.get_pixels()[i][j];
                    let j = j + slab.y_start;
                    let i = i + slab.x_start;

                    image.set_pixel(j, i, pixel);
                }
            }
        }
    }

    // for (j, row) in image.get_pixels_mut().iter_mut().enumerate() {
    //     for (i, pixel) in row.iter_mut().enumerate() {
    //         let j = height - j - 1;

    //         // use opengl coords, (0.0, 0.0) is center; (1.0, 1.0) is
    //         // top right; (-1.0, -1.0) is bottom left
    //         let u = ((i as Scalar / (width - 1) as Scalar) - 0.5) * 2.0;
    //         let v = ((j as Scalar / (height - 1) as Scalar) - 0.5) * 2.0;

    //         let ray = camera.get_ray(u, v);

    //         *pixel = trace_ray(&ray, &camera, &SCENE, 2);
    //     }
    // }

    let ppm = PPM::new(&image);
    ppm.write_to_file("image.ppm").unwrap();
}

use std::convert::TryInto;

use egui::{FontDefinitions, FontFamily, TextStyle};
use egui_glfw::EguiBackend;
use glfw::{Action, Context, Key};

use rt::drawable::Drawable;
use rt::fps::FPS;
use rt::gl_camera;
use rt::gpu_immediate::GPUImmediate;
use rt::mesh;
use rt::mesh::{MeshDrawData, MeshUseShader};
use rt::shader;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

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

    let mut camera = gl_camera::Camera::new(
        glm::vec3(0.0, 0.0, 3.0),
        glm::vec3(0.0, 1.0, 0.0),
        -90.0,
        0.0,
        45.0,
    );

    let mut imm = GPUImmediate::new();

    let directional_light_shader = shader::builtins::get_directional_light_shader()
        .as_ref()
        .unwrap();

    let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
        .as_ref()
        .unwrap();

    let face_orientation_shader = shader::builtins::get_face_orientation_shader()
        .as_ref()
        .unwrap();

    println!(
        "directional_light: uniforms: {:?} attributes: {:?}",
        directional_light_shader.get_uniforms(),
        directional_light_shader.get_attributes(),
    );

    println!(
        "smooth_color_3d: uniforms: {:?} attributes: {:?}",
        smooth_color_3d_shader.get_uniforms(),
        smooth_color_3d_shader.get_attributes(),
    );

    println!(
        "face_orientation: uniforms: {:?} attributes: {:?}",
        face_orientation_shader.get_uniforms(),
        face_orientation_shader.get_attributes(),
    );

    let mut last_cursor = window.get_cursor_pos();

    let mut fps = FPS::default();

    let mut draw_bvh = true;
    let mut bvh_draw_level = 0;
    let mut should_cast_ray = false;
    let mut bvh_ray_intersection = Vec::new();

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

        let projection_matrix =
            &glm::convert(camera.get_projection_matrix(window_width, window_height));
        let view_matrix = &glm::convert(camera.get_view_matrix());

        // Shader stuff
        {
            {
                directional_light_shader.use_shader();
                directional_light_shader.set_mat4("projection\0", projection_matrix);
                directional_light_shader.set_mat4("view\0", view_matrix);
                directional_light_shader.set_mat4("model\0", &glm::identity());
                directional_light_shader
                    .set_vec3("viewPos\0", &glm::convert(camera.get_position()));
                directional_light_shader.set_vec3("material.color\0", &glm::vec3(0.3, 0.2, 0.7));
                directional_light_shader.set_vec3("material.specular\0", &glm::vec3(0.3, 0.3, 0.3));
                directional_light_shader.set_float("material.shininess\0", 4.0);
                directional_light_shader
                    .set_vec3("light.direction\0", &glm::vec3(-0.7, -1.0, -0.7));
                directional_light_shader.set_vec3("light.ambient\0", &glm::vec3(0.3, 0.3, 0.3));
                directional_light_shader.set_vec3("light.diffuse\0", &glm::vec3(1.0, 1.0, 1.0));
                directional_light_shader.set_vec3("light.specular\0", &glm::vec3(1.0, 1.0, 1.0));
            }

            {
                smooth_color_3d_shader.use_shader();
                smooth_color_3d_shader.set_mat4("projection\0", projection_matrix);
                smooth_color_3d_shader.set_mat4("view\0", view_matrix);
                smooth_color_3d_shader.set_mat4("model\0", &glm::identity());
            }

            {
                face_orientation_shader.use_shader();
                face_orientation_shader.set_mat4("projection\0", projection_matrix);
                face_orientation_shader.set_mat4("view\0", view_matrix);
                face_orientation_shader.set_mat4("model\0", &glm::identity());
                face_orientation_shader
                    .set_vec4("color_face_front\0", &glm::vec4(0.0, 0.0, 1.0, 1.0));
                face_orientation_shader
                    .set_vec4("color_face_back\0", &glm::vec4(1.0, 0.0, 0.0, 1.0));
            }
        }

        unsafe {
            gl::Disable(gl::BLEND);
        }

        directional_light_shader.use_shader();
        directional_light_shader.set_mat4("model\0", &glm::identity());
        mesh.draw(&mut MeshDrawData::new(
            &mut imm,
            MeshUseShader::DirectionalLight,
            draw_bvh,
            bvh_draw_level,
            None,
        ))
        .unwrap();

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

                let format = imm.get_cleared_vertex_format();
                let pos_attr = format.add_attribute(
                    "in_pos\0".to_string(),
                    rt::gpu_immediate::GPUVertCompType::F32,
                    3,
                    rt::gpu_immediate::GPUVertFetchMode::Float,
                );
                let color_attr = format.add_attribute(
                    "in_color\0".to_string(),
                    rt::gpu_immediate::GPUVertCompType::F32,
                    4,
                    rt::gpu_immediate::GPUVertFetchMode::Float,
                );

                imm.begin(
                    rt::gpu_immediate::GPUPrimType::Lines,
                    bvh_ray_intersection.len() * 2,
                    smooth_color_3d_shader,
                );

                bvh_ray_intersection.iter().for_each(|(pos, ray_hit_info)| {
                    let p1: glm::Vec3 = glm::convert(*pos);
                    let p2: glm::Vec3 = glm::convert(ray_hit_info.data.as_ref().unwrap().co);

                    imm.attr_4f(color_attr, 0.8, 0.3, 0.8, 1.0);
                    imm.vertex_3f(pos_attr, p1[0], p1[1], p1[2]);
                    imm.attr_4f(color_attr, 0.8, 0.3, 0.8, 1.0);
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

            // GUI starts
            {
                egui.begin_frame(&window, &mut glfw);
                egui::Window::new("Hello world!").show(egui.get_egui_ctx(), |ui| {
                    ui.label("Hello RT Rust!");
                    ui.label(format!("fps: {:.2}", fps.update_and_get(Some(60.0))));

                    ui.checkbox(&mut draw_bvh, "Draw BVH");
                    ui.add(egui::Slider::new(&mut bvh_draw_level, 0..=15).text("BVH Draw Level"));
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
    camera: &mut gl_camera::Camera,
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
