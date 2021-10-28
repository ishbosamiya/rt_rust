use egui;

use crate::glm;

pub trait DrawUI {
    /// Draw UI for elements that do not change self
    fn draw_ui(&self, ui: &mut egui::Ui);

    /// Draw UI for elements that change self
    fn draw_ui_mut(&mut self, ui: &mut egui::Ui);
}

fn color_edit_dvec4(ui: &mut egui::Ui, color: &mut glm::DVec4) {
    let mut color_egui = [
        color[0] as f32,
        color[1] as f32,
        color[2] as f32,
        color[3] as f32,
    ];
    ui.color_edit_button_rgba_premultiplied(&mut color_egui);
    *color = glm::vec4(
        color_egui[0] as f64,
        color_egui[1] as f64,
        color_egui[2] as f64,
        color_egui[3] as f64,
    );
}

fn color_edit_dvec3(ui: &mut egui::Ui, color: &mut glm::DVec3) {
    let mut color_egui = [color[0] as f32, color[1] as f32, color[2] as f32];
    ui.color_edit_button_rgb(&mut color_egui);
    *color = glm::vec3(
        color_egui[0] as f64,
        color_egui[1] as f64,
        color_egui[2] as f64,
    );
}

pub fn color_edit_button_dvec4(ui: &mut egui::Ui, text: &str, color: &mut glm::DVec4) {
    ui.horizontal(|ui| {
        ui.label(text);
        color_edit_dvec4(ui, color);
    });
}

pub fn color_edit_button_dvec3(ui: &mut egui::Ui, text: &str, color: &mut glm::DVec3) {
    ui.horizontal(|ui| {
        ui.label(text);
        color_edit_dvec3(ui, color);
    });
}
