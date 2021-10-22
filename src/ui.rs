use egui;

pub trait DrawUI {
    /// Draw UI for elements that do not change self
    fn draw_ui(&self, ui: &mut egui::Ui);

    /// Draw UI for elements that change self
    fn draw_ui_mut(&mut self, ui: &mut egui::Ui);
}
