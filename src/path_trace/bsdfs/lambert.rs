use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use crate::math;
use crate::path_trace::medium::Medium;
use crate::ui::DrawUI;
use crate::{glm, ui};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Lambert {
    color: glm::DVec3,
}

impl Default for Lambert {
    fn default() -> Self {
        Self::new(glm::vec3(1.0, 1.0, 1.0))
    }
}

impl Lambert {
    pub fn new(color: glm::DVec3) -> Self {
        Self { color }
    }
}

#[typetag::serde]
impl BSDF for Lambert {
    fn sample(
        &self,
        _wo: &glm::DVec3,
        _wo_medium: &Medium,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData> {
        // TODO: make this random in hemisphere instead of using a
        // sphere for better performance
        if sampling_types.contains(SamplingTypes::Diffuse) {
            // need to return `wi` which should point towards the hitpoint
            Some(SampleData::new(
                -(intersect_info.get_normal().unwrap() + math::random_in_unit_sphere()),
                SamplingTypes::Diffuse,
            ))
        } else {
            None
        }
    }

    fn eval(
        &self,
        _wi: &glm::DVec3,
        _wo: &glm::DVec3,
        _wo_medium: &Medium,
        _intersect_info: &IntersectInfo,
    ) -> glm::DVec3 {
        #[allow(clippy::let_and_return)]
        self.color
    }

    fn get_bsdf_name(&self) -> &str {
        "Lambert"
    }

    fn get_base_color(&self) -> glm::DVec3 {
        self.color
    }
}

impl DrawUI for Lambert {
    fn draw_ui(&self, ui: &mut egui::Ui) {
        ui.label(format!("BSDF: {}", self.get_bsdf_name()));
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui) {
        ui::color_edit_button_dvec3(ui, "Base Color", &mut self.color);
    }
}
