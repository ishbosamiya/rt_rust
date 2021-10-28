use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use crate::path_trace::medium::Medium;
use crate::ui::DrawUI;
use crate::{glm, ui};

// TODO: add roughness parameter, right now it is purely reflective
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Refraction {
    color: glm::DVec3,
    ior: f64,
}

impl Default for Refraction {
    fn default() -> Self {
        Self::new(glm::vec3(1.0, 1.0, 1.0), 1.5)
    }
}

impl Refraction {
    pub fn new(color: glm::DVec3, ior: f64) -> Self {
        Self { color, ior }
    }
}

#[typetag::serde]
impl BSDF for Refraction {
    fn sample(
        &self,
        wo: &glm::DVec3,
        wo_medium: &Medium,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData> {
        // TODO: need to figure out which sampling type it would be,
        // both diffuse and reflection seem to make sense

        let ior = if intersect_info.get_front_face() {
            wo_medium.get_ior() / self.get_ior()
        } else {
            self.get_ior() / wo_medium.get_ior()
        };

        if sampling_types.contains(SamplingTypes::Diffuse) {
            let output =
                -glm::refract_vec(&-wo, intersect_info.get_normal().as_ref().unwrap(), ior);
            Some(SampleData::new(output, SamplingTypes::Diffuse))
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
        self.color
    }

    fn get_bsdf_name(&self) -> &str {
        "Refraction"
    }

    fn get_base_color(&self) -> glm::DVec3 {
        self.color
    }

    fn get_ior(&self) -> f64 {
        self.ior
    }
}

impl DrawUI for Refraction {
    fn draw_ui(&self, ui: &mut egui::Ui) {
        ui.label(format!("BSDF: {}", self.get_bsdf_name()));
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui) {
        ui::color_edit_button_dvec3(ui, "Base Color", &mut self.color);
        ui.add(egui::Slider::new(&mut self.ior, 1.0..=2.0).text("ior"));
    }
}
