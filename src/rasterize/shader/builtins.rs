//! Load builtin shaders

use lazy_static::lazy_static;
use paste::paste;
use quick_renderer::{
    camera::Camera,
    load_builtin_shader, load_builtin_shader_easy,
    shader::{
        builtins::{
            display_uniform_and_attribute_info as other_display_uniform_and_attribute_info,
            setup_shaders as other_setup_shaders,
        },
        Shader, ShaderError,
    },
};

use crate::glm;

/// Reexport already available builtin shaders
pub use quick_renderer::shader::builtins::*;

load_builtin_shader_easy!(
    environment;
    "../../../shaders/environment_shader.vert";
    "../../../shaders/environment_shader.frag"
);

load_builtin_shader_easy!(
    screen_texture;
    "../../../shaders/screen_texture.vert";
    "../../../shaders/screen_texture.frag"
);

pub fn display_uniform_and_attribute_info() {
    other_display_uniform_and_attribute_info();

    {
        let environment_shader = get_environment_shader().as_ref().unwrap();

        println!(
            "environment_shader: uniforms: {:?} attributes: {:?}",
            environment_shader.get_uniforms(),
            environment_shader.get_attributes(),
        );
    }

    {
        let screen_texture_shader = get_screen_texture_shader().as_ref().unwrap();

        println!(
            "screen_texture_shader: uniforms: {:?} attributes: {:?}",
            screen_texture_shader.get_uniforms(),
            screen_texture_shader.get_attributes(),
        );
    }
}

pub fn setup_shaders(camera: &Camera, window_width: usize, window_height: usize) {
    other_setup_shaders(camera, window_width, window_height);

    let projection_matrix =
        &glm::convert(camera.get_projection_matrix(window_width, window_height));
    let view_matrix = &glm::convert(camera.get_view_matrix());

    {
        let environment_shader = get_environment_shader().as_ref().unwrap();

        environment_shader.use_shader();
        environment_shader.set_mat4("projection\0", projection_matrix);
        environment_shader.set_mat4("view\0", view_matrix);
        environment_shader.set_mat4("model\0", &glm::identity());
    }

    {
        let screen_texture_shader = get_screen_texture_shader().as_ref().unwrap();

        screen_texture_shader.use_shader();
    }
}
