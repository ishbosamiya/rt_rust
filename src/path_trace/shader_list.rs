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
    /// Must give mutable access to `BSDF` that the `Self` contains
    fn get_bsdf_mut(&mut self) -> &mut dyn BSDF;
    /// Get the `ShaderID` assigned to the shader
    fn get_shader_id(&self) -> ShaderID;

    /// Get mutable reference to the name of the shader
    fn get_shader_name_mut(&mut self) -> &mut String;
    /// Get reference to the name of the shader
    fn get_shader_name(&self) -> &String;
}

pub struct ShaderList {
    /// list of all shaders indexed by their ShaderID
    shaders: HashMap<ShaderID, Box<dyn Shader>>,
    /// list of all shader ids in the order of addition of shaders
    shader_ids: Vec<ShaderID>,

    /// selected shader if there exists one
    selected_shader: Option<ShaderID>,
}

impl ShaderList {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
            shader_ids: Vec::new(),
            selected_shader: None,
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

    pub fn get_selected_shader(&self) -> &Option<ShaderID> {
        &self.selected_shader
    }

    pub fn deselect_shader(&mut self) {
        self.selected_shader = None;
    }

    pub fn add_shader(&mut self, mut shader: Box<dyn Shader>) -> ShaderID {
        let shader_id = ShaderID(rand::random());
        shader.set_shader_id(shader_id);
        self.shaders.insert(shader_id, shader);
        self.shader_ids.push(shader_id);
        shader_id
    }
}

impl Default for ShaderList {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawUI for ShaderList {
    fn draw_ui(&self, ui: &mut egui::Ui) {
        if let Some(selected_shader) = self.selected_shader {
            ui.label(format!(
                "Selected Shader: {}",
                self.shaders
                    .get(&selected_shader)
                    .unwrap()
                    .get_shader_name()
            ));
        } else {
            ui.label("No shader selected");
        }
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui) {
        assert_eq!(self.shader_ids.len(), self.shaders.len());
        let selected_shader = &mut self.selected_shader;
        let shaders = &mut self.shaders;
        ui.separator();
        self.shader_ids
            .iter()
            .enumerate()
            .for_each(|(index, shader_id)| {
                ui.label(format!("Shader {}", index + 1));

                let shader = shaders.get_mut(shader_id).unwrap();

                ui.horizontal(|ui| {
                    ui.text_edit_singleline(shader.get_shader_name_mut());
                    if ui.button("Select Shader").clicked() {
                        *selected_shader = Some(shader.get_shader_id());
                    }
                });

                shader.get_bsdf().draw_ui(ui);
                shader.get_bsdf_mut().draw_ui_mut(ui);

                ui.separator();
            });
    }
}
