use lazy_static::lazy_static;
use paste::paste;

use super::{Shader, ShaderError};
use crate::glm;
use crate::rasterize::gl_camera::Camera;

macro_rules! load_builtin_shader {
    ( $get_shader:ident ; $get_vert_code:ident ; $get_frag_code:ident ; $vert_location:tt ; $frag_location:tt ; $static_name:ident ) => {
        lazy_static! {
            static ref $static_name: Result<Shader, ShaderError> =
                { Shader::from_strings($get_vert_code(), $get_frag_code(),) };
        }

        pub fn $get_vert_code() -> &'static str {
            include_str!($vert_location)
        }

        pub fn $get_frag_code() -> &'static str {
            include_str!($frag_location)
        }

        pub fn $get_shader() -> &'static Result<Shader, ShaderError> {
            &$static_name
        }
    };
}

macro_rules! load_builtin_shader_easy {
    ( $name:ident ; $vert_location:tt ; $frag_location:tt ) => {
        paste! {
            load_builtin_shader!([<get_ $name _shader>]; [<get_ $name _vert_code>]; [<get_ $name _frag_code>]; $vert_location; $frag_location; [<$name:upper>]);
        }
    }
}

load_builtin_shader_easy!(
    directional_light;
    "../../../shaders/directional_light.vert";
    "../../../shaders/directional_light.frag"
);

load_builtin_shader_easy!(
    smooth_color_3d;
    "../../../shaders/shader_3D_smooth_color.vert";
    "../../../shaders/shader_3D_smooth_color.frag");

load_builtin_shader_easy!(
    face_orientation;
    "../../../shaders/face_orientation.vert";
    "../../../shaders/face_orientation.frag"
);

load_builtin_shader_easy!(
    flat_texture;
    "../../../shaders/flat_texture.vert";
    "../../../shaders/flat_texture.frag"
);

load_builtin_shader_easy!(
    smooth_sphere;
    "../../../shaders/smooth_sphere.vert";
    "../../../shaders/smooth_sphere.frag"
);

load_builtin_shader_easy!(
    infinite_grid;
    "../../../shaders/infinite_grid.vert";
    "../../../shaders/infinite_grid.frag"
);

pub fn display_uniform_and_attribute_info() {
    {
        let directional_light_shader = get_directional_light_shader().as_ref().unwrap();

        println!(
            "directional_light: uniforms: {:?} attributes: {:?}",
            directional_light_shader.get_uniforms(),
            directional_light_shader.get_attributes(),
        );
    }

    {
        let smooth_color_3d_shader = get_smooth_color_3d_shader().as_ref().unwrap();

        println!(
            "smooth_color_3d: uniforms: {:?} attributes: {:?}",
            smooth_color_3d_shader.get_uniforms(),
            smooth_color_3d_shader.get_attributes(),
        );
    }

    {
        let face_orientation_shader = get_face_orientation_shader().as_ref().unwrap();

        println!(
            "face_orientation: uniforms: {:?} attributes: {:?}",
            face_orientation_shader.get_uniforms(),
            face_orientation_shader.get_attributes(),
        );
    }

    {
        let flat_texture_shader = get_flat_texture_shader().as_ref().unwrap();

        println!(
            "flat_texture: uniforms: {:?} attributes: {:?}",
            flat_texture_shader.get_uniforms(),
            flat_texture_shader.get_attributes(),
        );
    }

    {
        let smooth_sphere_shader = get_smooth_sphere_shader().as_ref().unwrap();

        println!(
            "smooth_sphere: uniforms: {:?} attributes: {:?}",
            smooth_sphere_shader.get_uniforms(),
            smooth_sphere_shader.get_attributes(),
        );
    }

    {
        let infinite_grid_shader = get_infinite_grid_shader().as_ref().unwrap();

        println!(
            "smooth_sphere: uniforms: {:?} attributes: {:?}",
            infinite_grid_shader.get_uniforms(),
            infinite_grid_shader.get_attributes(),
        );
    }
}

pub fn setup_shaders(camera: &Camera, window_width: usize, window_height: usize) {
    let projection_matrix =
        &glm::convert(camera.get_projection_matrix(window_width, window_height));
    let view_matrix = &glm::convert(camera.get_view_matrix());

    {
        let directional_light_shader = get_directional_light_shader().as_ref().unwrap();

        directional_light_shader.use_shader();
        directional_light_shader.set_mat4("projection\0", projection_matrix);
        directional_light_shader.set_mat4("view\0", view_matrix);
        directional_light_shader.set_mat4("model\0", &glm::identity());
        directional_light_shader.set_vec3("viewPos\0", &glm::convert(camera.get_position()));
        directional_light_shader.set_vec3("material.color\0", &glm::vec3(0.3, 0.2, 0.7));
        directional_light_shader.set_vec3("material.specular\0", &glm::vec3(0.3, 0.3, 0.3));
        directional_light_shader.set_float("material.shininess\0", 4.0);
        directional_light_shader.set_vec3("light.direction\0", &glm::vec3(-0.7, -1.0, -0.7));
        directional_light_shader.set_vec3("light.ambient\0", &glm::vec3(0.3, 0.3, 0.3));
        directional_light_shader.set_vec3("light.diffuse\0", &glm::vec3(1.0, 1.0, 1.0));
        directional_light_shader.set_vec3("light.specular\0", &glm::vec3(1.0, 1.0, 1.0));
    }

    {
        let smooth_color_3d_shader = get_smooth_color_3d_shader().as_ref().unwrap();

        smooth_color_3d_shader.use_shader();
        smooth_color_3d_shader.set_mat4("projection\0", projection_matrix);
        smooth_color_3d_shader.set_mat4("view\0", view_matrix);
        smooth_color_3d_shader.set_mat4("model\0", &glm::identity());
    }

    {
        let face_orientation_shader = get_face_orientation_shader().as_ref().unwrap();

        face_orientation_shader.use_shader();
        face_orientation_shader.set_mat4("projection\0", projection_matrix);
        face_orientation_shader.set_mat4("view\0", view_matrix);
        face_orientation_shader.set_mat4("model\0", &glm::identity());
        face_orientation_shader.set_vec4("color_face_front\0", &glm::vec4(0.0, 0.0, 1.0, 1.0));
        face_orientation_shader.set_vec4("color_face_back\0", &glm::vec4(1.0, 0.0, 0.0, 1.0));
    }

    {
        let flat_texture_shader = get_flat_texture_shader().as_ref().unwrap();

        flat_texture_shader.use_shader();
        flat_texture_shader.set_mat4("projection\0", projection_matrix);
        flat_texture_shader.set_mat4("view\0", view_matrix);
        flat_texture_shader.set_mat4("model\0", &glm::identity());
    }

    {
        let smooth_sphere_shader = get_smooth_sphere_shader().as_ref().unwrap();

        smooth_sphere_shader.use_shader();
        smooth_sphere_shader.set_mat4("projection\0", projection_matrix);
        smooth_sphere_shader.set_mat4("view\0", view_matrix);
    }

    {
        let infinite_grid_shader = get_infinite_grid_shader().as_ref().unwrap();

        infinite_grid_shader.use_shader();
        infinite_grid_shader.set_mat4("projection\0", projection_matrix);
        infinite_grid_shader.set_mat4("view\0", view_matrix);
    }
}
