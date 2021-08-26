use lazy_static::lazy_static;
use paste::paste;

use super::{Shader, ShaderError};

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
    "../../shaders/directional_light.vert";
    "../../shaders/directional_light.frag"
);

load_builtin_shader_easy!(
    smooth_color_3d;
    "../../shaders/shader_3D_smooth_color.vert";
    "../../shaders/shader_3D_smooth_color.frag");

load_builtin_shader_easy!(
    face_orientation;
    "../../shaders/face_orientation.vert";
    "../../shaders/face_orientation.frag"
);
