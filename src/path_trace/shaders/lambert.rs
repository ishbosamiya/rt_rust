use crate::path_trace::{
    bsdf::BSDF,
    bsdfs::lambert::Lambert as LambertBSDF,
    shader_list::{Shader, ShaderID},
};

pub struct Lambert {
    bsdf: LambertBSDF,
    shader_id: Option<ShaderID>,
}

impl Lambert {
    pub fn new(bsdf: LambertBSDF) -> Self {
        Self {
            bsdf,
            shader_id: None,
        }
    }
}

impl Shader for Lambert {
    fn set_shader_id(&mut self, shader_id: ShaderID) {
        self.shader_id = Some(shader_id);
    }

    fn get_bsdf(&self) -> &dyn BSDF {
        &self.bsdf
    }

    fn get_shader_id(&self) -> ShaderID {
        self.shader_id.unwrap()
    }
}
