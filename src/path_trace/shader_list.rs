use std::collections::HashMap;

use super::bsdf::BSDF;

/// A unique identifier given to each `Shader` during its
/// initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShaderID(usize);

pub trait Shader: Sync + Send {
    /// Set the `ShaderID`, can be requested for later using
    /// `get_shader_id()`
    fn set_shader_id(&mut self, shader_id: ShaderID);
    /// Must give access to `BSDF` that the `Self` contains
    fn get_bsdf(&self) -> &dyn BSDF;
    /// Get the `ShaderID` assigned to the shader
    fn get_shader_id(&self) -> ShaderID;
}

pub struct ShaderList {
    shaders: HashMap<ShaderID, Box<dyn Shader>>,
}

impl ShaderList {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
        }
    }

    pub fn get_shaders(&self) -> &HashMap<ShaderID, Box<dyn Shader>> {
        &self.shaders
    }

    pub fn get_shader(&self, shader_id: ShaderID) -> Option<&dyn Shader> {
        self.shaders
            .get(&shader_id)
            .map(|boxed_shader| boxed_shader.as_ref())
    }

    pub fn add_shader(&mut self, mut shader: Box<dyn Shader>) -> ShaderID {
        let shader_id = ShaderID(rand::random());
        shader.set_shader_id(shader_id);
        self.shaders.insert(shader_id, shader);
        shader_id
    }
}

impl Default for ShaderList {
    fn default() -> Self {
        Self::new()
    }
}
