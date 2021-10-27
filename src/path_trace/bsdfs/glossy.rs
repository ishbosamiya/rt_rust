use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use crate::ui::DrawUI;
use crate::{glm, ui};

// TODO: add roughness parameter, right now it is purely reflective
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Glossy {
    color: glm::DVec4,
}

impl Default for Glossy {
    fn default() -> Self {
        Self::new(glm::vec4(1.0, 1.0, 1.0, 1.0))
    }
}

impl Glossy {
    pub fn new(color: glm::DVec4) -> Self {
        Self { color }
    }
}

#[typetag::serde]
impl BSDF for Glossy {
    fn sample(
        &self,
        wo: &glm::DVec3,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData> {
        if sampling_types.contains(SamplingTypes::Reflection) {
            // need to consider the inverse of the outgoing direction
            // during reflection
            Some(SampleData::new(
                glm::reflect_vec(&-wo, intersect_info.get_normal().as_ref().unwrap()),
                SamplingTypes::Reflection,
            ))
        } else {
            None
        }
    }

    fn eval(
        &self,
        _wi: &glm::DVec3,
        _wo: &glm::DVec3,
        _intersect_info: &IntersectInfo,
    ) -> glm::DVec3 {
        #[allow(clippy::let_and_return)]
        let color = glm::vec4_to_vec3(&self.color);

        color
    }

    fn get_bsdf_name(&self) -> &str {
        "Glossy"
    }

    fn get_base_color(&self) -> glm::DVec3 {
        glm::vec4_to_vec3(&self.color)
    }
}

impl DrawUI for Glossy {
    fn draw_ui(&self, ui: &mut egui::Ui) {
        ui.label(format!("BSDF: {}", self.get_bsdf_name()));
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui) {
        ui::color_edit_button_dvec4(ui, "Base Color", &mut self.color);
    }
}
