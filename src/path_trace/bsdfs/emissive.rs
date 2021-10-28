use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use crate::path_trace::medium::Medium;
use crate::ui::DrawUI;
use crate::{glm, ui};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Emissive {
    color: glm::DVec4,
    power: f64,
}

impl Default for Emissive {
    fn default() -> Self {
        Self::new(glm::vec4(1.0, 1.0, 1.0, 1.0), 1.0)
    }
}

impl Emissive {
    pub fn new(color: glm::DVec4, power: f64) -> Self {
        Self { color, power }
    }
}

#[typetag::serde]
impl BSDF for Emissive {
    fn sample(
        &self,
        _wo: &glm::DVec3,
        _wo_medium: &Medium,
        _intersect_info: &IntersectInfo,
        _sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData> {
        None
    }

    fn eval(
        &self,
        _wi: &glm::DVec3,
        _wo: &glm::DVec3,
        _wo_medium: &Medium,
        _intersect_info: &IntersectInfo,
    ) -> glm::DVec3 {
        unreachable!("Emissive only material, so no eval is possible")
    }

    fn emission(&self, _intersect_info: &IntersectInfo) -> Option<glm::DVec3> {
        Some(glm::vec4_to_vec3(&(self.power * self.color)))
    }

    fn get_bsdf_name(&self) -> &str {
        "Emissive"
    }

    fn get_base_color(&self) -> glm::DVec3 {
        glm::vec4_to_vec3(&self.color)
    }
}

impl DrawUI for Emissive {
    fn draw_ui(&self, ui: &mut egui::Ui) {
        ui.label(format!("BSDF: {}", self.get_bsdf_name()));
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui) {
        ui::color_edit_button_dvec4(ui, "Base Color", &mut self.color);
        ui.add(egui::Slider::new(&mut self.power, 0.0..=10.0).text("Power"));
    }
}
