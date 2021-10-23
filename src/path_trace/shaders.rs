use std::fmt::Display;
use std::sync::Mutex;

use lazy_static::lazy_static;

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
    ( $shader_name:ident ; $bsdf:ty ) => {
        pub struct $shader_name {
            bsdf: $bsdf,
            shader_id: Option<ShaderID>,
            name: String,
        }

        impl $shader_name {
            pub fn new(bsdf: $bsdf) -> Self {
                Self {
                    bsdf,
                    shader_id: None,
                    name: SHADER_NAME_GEN.lock().unwrap().next().unwrap(),
                }
            }
        }

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
        }
    };
}

macro_rules! GenerateShaderTypes {
    ( $( $shader_type:ident ), *) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub enum ShaderType {
            $(
                $shader_type,
            )*
        }

        impl ShaderType {
            pub fn all() -> impl Iterator<Item = Self> {
                [
                    $(
                        Self::$shader_type,
                    )*
                ]
                    .iter()
                    .copied()
            }

            pub fn generate_shader(&self) -> Box<dyn Shader> {
                match self {
                    $(
                        Self::$shader_type => Box::new($shader_type::default()),
                    )*
                }
            }
        }

        impl Display for ShaderType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$shader_type => write!(f, stringify!($shader_type)),
                    )*
                }
            }
        }
    };
}

ShaderFromBSDF!(Lambert; bsdfs::lambert::Lambert);
ShaderFromBSDF!(Glossy; bsdfs::glossy::Glossy);
ShaderFromBSDF!(Emissive; bsdfs::emissive::Emissive);

GenerateShaderTypes!(Lambert, Glossy, Emissive);

impl Default for ShaderType {
    fn default() -> Self {
        Self::Lambert
    }
}
