use crate::glm;
use crate::ui::DrawUI;

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
    fn draw_ui(&self, ui: &mut egui::Ui) {
        ui.label("Transformations");
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui) {
        // For Location
        ui.add(egui::Slider::new(&mut self.location[0], -10.0..=10.0).text("Location-X"));
        ui.add(egui::Slider::new(&mut self.location[1], -10.0..=10.0).text("Location-Y"));
        ui.add(egui::Slider::new(&mut self.location[2], -10.0..=10.0).text("Location-Z"));

        // For Rotation
        ui.add(egui::Slider::new(&mut self.rotation[0], 0.0..=360.0).text("Rotation-X"));
        ui.add(egui::Slider::new(&mut self.rotation[1], 0.0..=360.0).text("Rotation-Y"));
        ui.add(egui::Slider::new(&mut self.rotation[2], 0.0..=360.0).text("Rotation-Z"));

        // For Scale
        ui.add(egui::Slider::new(&mut self.scale[0], 0.0..=10.0).text("Scale-X"));
        ui.add(egui::Slider::new(&mut self.scale[1], 0.0..=10.0).text("Scale-Y"));
        ui.add(egui::Slider::new(&mut self.scale[2], 0.0..=10.0).text("Scale-Z"));
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
