use enumflags2::BitFlags;

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use crate::math;
use crate::ui::DrawUI;
use crate::{glm, ui};

pub struct Lambert {
    color: glm::DVec4,
}

impl Lambert {
    pub fn new(color: glm::DVec4) -> Self {
        Self { color }
    }
}

impl BSDF for Lambert {
    fn sample(
        &self,
        _wo: &glm::DVec3,
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
        _intersect_info: &IntersectInfo,
    ) -> glm::DVec3 {
        #[allow(clippy::let_and_return)]
        let color = glm::vec4_to_vec3(&self.color);

        color
    }

    fn get_bsdf_name(&self) -> &str {
        "Lambert"
    }
}

impl DrawUI for Lambert {
    fn draw_ui(&self, ui: &mut egui::Ui) {
        ui.label(format!("BSDF: {}", self.get_bsdf_name()));
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui) {
        ui::color_edit_button_dvec4(ui, "Base Color", &mut self.color);
    }
}
