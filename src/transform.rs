use crate::ui::DrawUI;
use crate::{glm, UiData};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Transform {
    /// Location (in meters)
    pub location: glm::DVec3,
    /// Rotation (in degrees)
    pub rotation: glm::DVec3,
    /// Scale
    pub scale: glm::DVec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            location: glm::zero(),
            rotation: glm::zero(),
            scale: glm::vec3(1.0, 1.0, 1.0),
        }
    }
}

impl DrawUI for Transform {
    type ExtraData = UiData;

    fn draw_ui(&self, _ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {}

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {
        // For Location
        ui.collapsing("Location", |ui| {
            ui.add(egui::Slider::new(&mut self.location[0], -10.0..=10.0).text("X"));
            ui.add(egui::Slider::new(&mut self.location[1], -10.0..=10.0).text("Y"));
            ui.add(egui::Slider::new(&mut self.location[2], -10.0..=10.0).text("Z"));
        });

        // For Rotation
        ui.collapsing("Rotation", |ui| {
            ui.add(egui::Slider::new(&mut self.rotation[0], 0.0..=360.0).text("X"));
            ui.add(egui::Slider::new(&mut self.rotation[1], 0.0..=360.0).text("Y"));
            ui.add(egui::Slider::new(&mut self.rotation[2], 0.0..=360.0).text("Z"));
        });

        // For Scale
        ui.collapsing("Scaling", |ui| {
            ui.add(egui::Slider::new(&mut self.scale[0], 0.0..=10.0).text("X"));
            ui.add(egui::Slider::new(&mut self.scale[1], 0.0..=10.0).text("Y"));
            ui.add(egui::Slider::new(&mut self.scale[2], 0.0..=10.0).text("Z"));
        });
    }
}

impl Transform {
    pub fn new(location: glm::DVec3, rotation: glm::DVec3, scale: glm::DVec3) -> Self {
        Self {
            location,
            rotation,
            scale,
        }
    }

    pub fn get_matrix(&self) -> glm::DMat4 {
        let translated_mat = glm::translate(&glm::identity(), &self.location);
        let rotated_mat = glm::rotate_z(
            &glm::rotate_y(
                &glm::rotate_x(&translated_mat, self.rotation[0].to_radians()),
                self.rotation[1].to_radians(),
            ),
            self.rotation[2].to_radians(),
        );

        glm::scale(&rotated_mat, &self.scale)
    }
}
