use std::{cell::RefCell, convert::TryInto, rc::Rc};

use egui_glfw::{
    egui::{self, FontDefinitions, FontFamily, TextStyle},
    EguiBackend,
};
use glfw::{Action, Context, Key};

use quick_renderer::{
    camera::Camera,
    drawable::Drawable,
    gpu_immediate::GPUImmediate,
    infinite_grid::{InfiniteGrid, InfiniteGridDrawData},
};
use rt::{fps::FPS, glm, rasterize::shader, ui, viewport::Viewport};

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

    let mut use_top_panel = false;
    let mut use_bottom_panel = false;
    let mut use_left_panel = true;
    let mut use_right_panel = false;
    let mut background_color = glm::vec4(0.051, 0.051, 0.051, 1.0);

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
        {}
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
                .draw(&InfiniteGridDrawData::new(
                    imm.clone(),
                    glm::vec4(0.2, 0.2, 0.2, 1.0),
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

        // Swap front and back buffers
        window.swap_buffers();
    }
}

#[allow(clippy::too_many_arguments)]
fn quick_test_handle_event(
    event: &glfw::WindowEvent,
    window: &mut glfw::Window,
    camera: &mut Camera,
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
