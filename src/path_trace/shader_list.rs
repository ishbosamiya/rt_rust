use std::collections::HashMap;

use crate::ui::DrawUI;

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

    /// Get mutable reference to the name of the shader
    fn get_shader_name_mut(&mut self) -> &mut String;
    /// Get reference to the name of the shader
    fn get_shader_name(&self) -> &String;
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

impl DrawUI for ShaderList {
    fn draw_ui(&self, _ui: &mut egui::Ui) {}

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui) {
        self.shaders
            .values_mut()
            .enumerate()
            .for_each(|(index, shader)| {
                ui.horizontal(|ui| {
                    ui.label(format!("Shader {}", index));
                    ui.text_edit_singleline(shader.get_shader_name_mut());
                });

                ui.separator();
            });
    }
}
