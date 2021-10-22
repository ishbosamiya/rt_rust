use crate::path_trace::{
    bsdf::BSDF,
    bsdfs,
    shader_list::{Shader, ShaderID},
};

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
                    name: "No Name Assigned".to_string(),
                }
            }
        }

        impl Shader for $shader_name {
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

ShaderFromBSDF!(Lambert; bsdfs::lambert::Lambert);
ShaderFromBSDF!(Glossy; bsdfs::glossy::Glossy);
ShaderFromBSDF!(Emissive; bsdfs::emissive::Emissive);
