use std::fmt::Display;
use std::sync::Mutex;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::glm;
use crate::path_trace::{
    bsdf::BSDF,
    bsdfs,
    shader_list::{Shader, ShaderID},
};

lazy_static! {
    static ref SHADER_NAME_GEN: Mutex<NameGen> = Mutex::new(NameGen::new("shader".to_string()));
}

struct NameGen {
    prefix: String,
    current_gen: usize,
}

impl NameGen {
    fn new(prefix: String) -> Self {
        Self {
            prefix,
            current_gen: 0,
        }
    }
}

impl Iterator for NameGen {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_gen += 1;

        Some(format!("{}_{}", self.prefix, self.current_gen))
    }
}

macro_rules! ShaderFromBSDF {
    ( $default_viewport_color:literal, $( $shader_name:ident, $bsdf:ty ); *) => {
        $(
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct $shader_name {
                bsdf: $bsdf,
                shader_id: Option<ShaderID>,
                name: String,

                #[serde(default = $default_viewport_color)]
                viewport_color: glm::DVec3,
            }

            impl $shader_name {
                pub fn new(bsdf: $bsdf) -> Self {
                    Self {
                        bsdf,
                        shader_id: None,
                        name: SHADER_NAME_GEN.lock().unwrap().next().unwrap(),
                        viewport_color: bsdf.get_base_color(),
                    }
                }
            }

            #[typetag::serde]
            impl Shader for $shader_name {
                fn default() -> Self
                where
                    Self: Sized,
                {
                    Self::new(Default::default())
                }

                fn set_shader_id(&mut self, shader_id: ShaderID) {
                    self.shader_id = Some(shader_id);
                }

                fn get_bsdf(&self) -> &dyn BSDF {
                    &self.bsdf
                }

                fn get_bsdf_mut(&mut self) -> &mut dyn BSDF {
                    &mut self.bsdf
                }

                fn get_shader_id(&self) -> ShaderID {
                    self.shader_id.unwrap()
                }

                fn get_shader_name_mut(&mut self) -> &mut String {
                    &mut self.name
                }

                fn get_shader_name(&self) -> &String {
                    &self.name
                }

                fn get_viewport_color(&self) -> &glm::DVec3 {
                    &self.viewport_color
                }

                fn get_viewport_color_mut(&mut self) -> &mut glm::DVec3 {
                    &mut self.viewport_color
                }
            }
        )*

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
	pub enum ShaderType {
            $(
                $shader_name,
            )*
        }

        impl ShaderType {
            pub fn all() -> impl Iterator<Item = Self> {
                [
                    $(
                        Self::$shader_name,
                    )*
                ]
                    .iter()
                    .copied()
            }

            pub fn generate_shader(&self) -> Box<dyn Shader> {
                match self {
                    $(
                        Self::$shader_name => Box::new($shader_name::default()),
                    )*
                }
            }
        }

        impl Display for ShaderType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$shader_name => write!(f, stringify!($shader_name)),
                    )*
                }
            }
        }
    };
}

fn default_viewport_color() -> glm::DVec3 {
    glm::zero()
}

ShaderFromBSDF!("default_viewport_color",
                Lambert, bsdfs::lambert::Lambert;
                Glossy, bsdfs::glossy::Glossy;
                Emissive, bsdfs::emissive::Emissive;
                Blinnphong, bsdfs::blinnphong::Blinnphong;
                Refraction, bsdfs::refraction::Refraction);

impl Default for ShaderType {
    fn default() -> Self {
        Self::Lambert
    }
}
