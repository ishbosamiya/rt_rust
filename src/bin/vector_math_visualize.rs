use std::{cell::RefCell, convert::TryInto, rc::Rc};

use egui::{FontDefinitions, FontFamily, TextStyle};
use egui_glfw::EguiBackend;
use glfw::{Action, Context, Key};

use rt::{
    fps::FPS,
    glm,
    rasterize::{
        camera::Camera as RasterizeCamera,
        drawable::Drawable,
        gpu_immediate::GPUImmediate,
        gpu_utils,
        infinite_grid::{InfiniteGrid, InfiniteGridDrawData},
        shader,
    },
    ui,
    viewport::Viewport,
};

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

    let mut camera = RasterizeCamera::new(
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

    let mut use_top_panel = false;
    let mut use_bottom_panel = false;
    let mut use_left_panel = true;
    let mut use_right_panel = false;
    let mut background_color = glm::vec4(0.051, 0.051, 0.051, 1.0);

    let mut wo_spherical = glm::vec2(30.0, 90.0);
    let mut normal_spherical = glm::vec2(90.0, 90.0);
    let mut wo_color = glm::vec4(1.0, 0.0, 0.0, 1.0);
    let mut normal_color = glm::vec4(1.0, 1.0, 1.0, 1.0);
    let mut refract_color = glm::vec4(0.0, 1.0, 0.0, 1.0);
    let mut reflect_color = glm::vec4(0.0, 0.0, 1.0, 1.0);

    let mut ior = 1.5;
    let mut refract_invert_wo = false;
    let mut refract_invert_output = false;
    let mut refract_invert_ior = false;
    let mut refract_invert_normal = false;

    while !window.should_close() {
        glfw.poll_events();

        glfw::flush_messages(&events).for_each(|(_, event)| {
            egui.handle_event(&event, &window);

            quick_test_handle_event(
                &event,
                &mut window,
                &mut camera,
                &mut window_last_cursor,
                &mut use_top_panel,
                &mut use_bottom_panel,
                &mut use_left_panel,
                &mut use_right_panel,
            );
        });

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
                    .min_width(0.1 * window_viewport.get_width() as f32)
                    .resizable(true)
                    .show(egui.get_egui_ctx(), |_ui| {})
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
                    .min_width(0.1 * window_viewport.get_width() as f32)
                    .resizable(true)
                    .show(egui.get_egui_ctx(), |ui| {
                        ui.label(format!("fps: {:.2}", fps.update_and_get(Some(60.0))));
                        ui::color_edit_button_dvec4(ui, "Background Color", &mut background_color);
                        ui.separator();

                        point_on_sphere_gui(ui, "wo", &mut wo_spherical);
                        point_on_sphere_gui(ui, "normal", &mut normal_spherical);

                        ui::color_edit_button_dvec4(ui, "wo Color", &mut wo_color);
                        ui::color_edit_button_dvec4(ui, "Normal Color", &mut normal_color);

                        ui.separator();

                        ui::color_edit_button_dvec4(ui, "Refract Color", &mut refract_color);
                        ui.checkbox(&mut refract_invert_wo, "Refract invert wo");
                        ui.checkbox(&mut refract_invert_output, "Refract invert output");
                        ui.checkbox(&mut refract_invert_ior, "Refract invert ior");
                        ui.checkbox(&mut refract_invert_normal, "Refract invert normal");
                        ui.add(egui::Slider::new(&mut ior, 0.0..=3.0).text("ior"));

                        ui::color_edit_button_dvec4(ui, "Reflect Color", &mut reflect_color);
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
        }
        // GUI Ends

        let _scene_last_cursor_pos = scene_viewport.calculate_location((
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

        // START: Draw all opaque objects
        {
            // vectors visualized
            {
                let wo = spherical_to_cartesian(&wo_spherical);
                let normal = spherical_to_cartesian(&normal_spherical);
                let origin = glm::vec3(1.0, 0.0, 0.0);

                // origin to wo
                draw_arrow(origin, wo + origin, wo_color, &mut imm.borrow_mut());

                // normal
                draw_arrow(origin, normal + origin, normal_color, &mut imm.borrow_mut());

                // reflect vector
                {
                    let reflected = -glm::reflect_vec(&-wo, &normal);

                    draw_arrow(
                        origin - reflected,
                        origin,
                        reflect_color,
                        &mut imm.borrow_mut(),
                    );
                }

                // refract vector
                {
                    let ior = if refract_invert_ior { 1.0 / ior } else { ior };
                    let wo = if refract_invert_wo { -wo } else { wo };
                    let normal = if refract_invert_normal {
                        -normal
                    } else {
                        normal
                    };

                    let output = refract_vec(&wo, &normal, ior);
                    let refracted = if refract_invert_output {
                        -output
                    } else {
                        output
                    };

                    draw_arrow(
                        origin - refracted,
                        origin,
                        refract_color,
                        &mut imm.borrow_mut(),
                    );
                }
            }
        }
        // END: Draw all opaque objects

        // Keep meshes that have shaders that need alpha channel
        // (blending) below this and handle it properly
        {
            unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            }

            // drawing the infinite grid
            infinite_grid
                .draw(&mut InfiniteGridDrawData::new(imm.clone()))
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

        // Swap front and back buffers
        window.swap_buffers();
    }
}

#[allow(clippy::too_many_arguments)]
fn quick_test_handle_event(
    event: &glfw::WindowEvent,
    window: &mut glfw::Window,
    camera: &mut RasterizeCamera,
    window_last_cursor: &mut (f64, f64),
    use_top_panel: &mut bool,
    use_bottom_panel: &mut bool,
    use_left_panel: &mut bool,
    use_right_panel: &mut bool,
) {
    let window_cursor = window.get_cursor_pos();

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
                *camera = RasterizeCamera::new(
                    glm::vec3(0.0, 0.0, -camera.get_position().norm()),
                    *camera.get_world_up(),
                    90.0,
                    0.0,
                    camera.get_fov(),
                    camera.get_sensor_no_ref(),
                )
            } else if modifier.contains(glfw::Modifiers::Alt) {
                *camera = RasterizeCamera::new(
                    glm::vec3(0.0, 0.0, camera.get_position().norm()),
                    *camera.get_world_up(),
                    -90.0,
                    0.0,
                    camera.get_fov(),
                    camera.get_sensor_no_ref(),
                )
            }
        }
        glfw::WindowEvent::Key(Key::Num3 | Key::Kp3, _, Action::Press, modifier) => {
            if modifier.contains(glfw::Modifiers::Control | glfw::Modifiers::Alt) {
                *camera = RasterizeCamera::new(
                    glm::vec3(-camera.get_position().norm(), 0.0, 0.0),
                    *camera.get_world_up(),
                    0.0,
                    0.0,
                    camera.get_fov(),
                    camera.get_sensor_no_ref(),
                )
            } else if modifier.contains(glfw::Modifiers::Alt) {
                *camera = RasterizeCamera::new(
                    glm::vec3(camera.get_position().norm(), 0.0, 0.0),
                    *camera.get_world_up(),
                    180.0,
                    0.0,
                    camera.get_fov(),
                    camera.get_sensor_no_ref(),
                )
            }
        }
        glfw::WindowEvent::Key(Key::Num7 | Key::Kp7, _, Action::Press, modifier) => {
            if modifier.contains(glfw::Modifiers::Control | glfw::Modifiers::Alt) {
                *camera = RasterizeCamera::new(
                    glm::vec3(0.0, -camera.get_position().norm(), 0.0),
                    *camera.get_world_up(),
                    -90.0,
                    90.0,
                    camera.get_fov(),
                    camera.get_sensor_no_ref(),
                )
            } else if modifier.contains(glfw::Modifiers::Alt) {
                *camera = RasterizeCamera::new(
                    glm::vec3(0.0, camera.get_position().norm(), 0.0),
                    *camera.get_world_up(),
                    -90.0,
                    -90.0,
                    camera.get_fov(),
                    camera.get_sensor_no_ref(),
                )
            }
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
                camera.get_fov(),
                camera.get_sensor_no_ref(),
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
    }

    *window_last_cursor = window_cursor;
}

fn refract_vec(i: &glm::DVec3, n: &glm::DVec3, eta: f64) -> glm::DVec3 {
    glm::refract_vec(i, n, eta)
}

// draw arrow from p1 to p2
fn draw_arrow(p1: glm::DVec3, p2: glm::DVec3, color: glm::DVec4, imm: &mut GPUImmediate) {
    gpu_utils::draw_smooth_sphere_at(p2, 0.03, glm::convert(color), glm::convert(color), imm);

    let p1: glm::Vec3 = glm::convert(p1);
    let p2: glm::Vec3 = glm::convert(p2);
    let color: glm::Vec4 = glm::convert(color);

    let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
        .as_ref()
        .unwrap();
    smooth_color_3d_shader.use_shader();
    smooth_color_3d_shader.set_mat4("model\0", &glm::identity());

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
        2,
        smooth_color_3d_shader,
    );

    imm.attr_4f(color_attr, color[0], color[1], color[2], 1.0);
    imm.vertex_3f(pos_attr, p1[0], p1[1], p1[2]);

    imm.attr_4f(color_attr, color[0], color[1], color[2], 1.0);
    imm.vertex_3f(pos_attr, p2[0], p2[1], p2[2]);

    imm.end();
}

fn _vec3_gui_edit(ui: &mut egui::Ui, text: &str, data: &mut glm::DVec3) {
    ui.label(text);
    ui.add(egui::Slider::new(&mut data[0], -5.0..=5.0));
    ui.add(egui::Slider::new(&mut data[1], -5.0..=5.0));
    ui.add(egui::Slider::new(&mut data[2], -5.0..=5.0));
}

fn point_on_sphere_gui(ui: &mut egui::Ui, text: &str, spherical: &mut glm::DVec2) {
    ui.label(text);

    ui.add(
        egui::Slider::new(&mut spherical[0], 0.0..=360.0)
            .clamp_to_range(true)
            .text("phi"),
    );
    ui.add(
        egui::Slider::new(&mut spherical[1], 0.0..=180.0)
            .clamp_to_range(true)
            .text("theta"),
    );
}

fn spherical_to_cartesian(spherical: &glm::DVec2) -> glm::DVec3 {
    let phi = spherical[0].to_radians();
    let theta = spherical[1].to_radians();
    glm::vec3(
        phi.cos() * theta.sin(),
        phi.sin() * theta.sin(),
        theta.cos(),
    )
}
