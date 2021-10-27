use std::{
    collections::{hash_map, HashMap},
    fmt::Debug,
};

use crate::ui::DrawUI;
use crate::{glm, ui};

use super::bsdf::BSDF;
use super::shaders::ShaderType;

use serde::{Deserialize, Serialize};

/// A unique identifier given to each `Shader` during its
/// initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ShaderID(usize);

#[typetag::serde(tag = "type")]
pub trait Shader: Debug + Sync + Send {
    fn default() -> Self
    where
        Self: Sized;

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

    /// Get reference shader's viewport color for the object
    fn get_viewport_color(&self) -> &glm::DVec3;
    /// Get mutable reference to shader's viewport color for the
    /// object
    fn get_viewport_color_mut(&mut self) -> &mut glm::DVec3;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShaderList {
    /// list of all shaders indexed by their ShaderID
    shaders: HashMap<ShaderID, Box<dyn Shader>>,
    /// list of all shader ids in the order of addition of shaders
    shader_ids: Vec<ShaderID>,

    /// selected shader if there exists one
    selected_shader: Option<ShaderID>,
    /// current shader type for adding new shader
    shader_type_for_add: ShaderType,
}

impl ShaderList {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
            shader_ids: Vec::new(),
            selected_shader: None,
            shader_type_for_add: ShaderType::default(),
        }
    }

    pub fn get_shaders(&self) -> hash_map::Values<'_, ShaderID, Box<dyn Shader>> {
        self.shaders.values()
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

    pub fn delete_shader(&mut self, shader_id: ShaderID) {
        self.shader_ids.remove(
            self.shader_ids
                .iter()
                .enumerate()
                .find(|(_, id)| shader_id == **id)
                .unwrap()
                .0,
        );

        self.shaders.remove(&shader_id).unwrap();
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
        assert_eq!(self.shader_ids.len(), self.shaders.len());

        if let Some(selected_shader) = self.selected_shader {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!(
                    "Selected Shader: {}",
                    match self.shaders.get(&selected_shader) {
                        Some(shader) => {
                            shader.get_shader_name()
                        }
                        None => "Not available",
                    }
                ));
                if ui.button("Deselect").clicked() {
                    self.deselect_shader();
                }
            });
        } else {
            ui.label("No shader selected");
        }

        let selected_shader = &mut self.selected_shader;
        let shaders = &mut self.shaders;
        let mut delete_shader = None;
        ui.separator();
        self.shader_ids
            .iter()
            .enumerate()
            .for_each(|(index, shader_id)| {
                let shader = shaders.get_mut(shader_id).unwrap();

                ui.horizontal(|ui| {
                    ui.label(format!("Shader {}", index + 1));
                    if ui.button("Select Shader").clicked() {
                        *selected_shader = Some(shader.get_shader_id());
                    }
                    if ui.button("X").clicked() {
                        delete_shader = Some(shader.get_shader_id());
                    }
                });

                ui.text_edit_singleline(shader.get_shader_name_mut());

                shader.get_bsdf().draw_ui(ui);
                shader.get_bsdf_mut().draw_ui_mut(ui);

                ui.horizontal_wrapped(|ui| {
                    ui::color_edit_button_dvec3(
                        ui,
                        "Viewport Color",
                        shader.get_viewport_color_mut(),
                    );

                    if ui.button("From BSDF").clicked() {
                        *shader.get_viewport_color_mut() = shader.get_bsdf().get_base_color();
                    }
                });

                ui.separator();
            });

        if let Some(shader_id) = delete_shader {
            self.delete_shader(shader_id);
        }

        ui.horizontal_wrapped(|ui| {
            egui::ComboBox::from_id_source(egui::Id::new("Shader Type"))
                .selected_text(format!("{}", self.shader_type_for_add))
                .show_ui(ui, |ui| {
                    ShaderType::all().for_each(|shader_type| {
                        ui.selectable_value(
                            &mut self.shader_type_for_add,
                            shader_type,
                            format!("{}", shader_type),
                        );
                    });
                });

            if ui.button("Add Shader").clicked() {
                self.add_shader(self.shader_type_for_add.generate_shader());
            }
        });
    }
}
