use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use crate::math;
use crate::ui::DrawUI;
use crate::{glm, ui};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Blinnphong {
    color: glm::DVec4,
    n: f64,
    divide_by_n_dot_l: bool,
}

impl Default for Blinnphong {
    fn default() -> Self {
        Self::new(glm::vec4(1.0, 1.0, 1.0, 1.0), 100.0, false)
    }
}

impl Blinnphong {
    pub fn new(color: glm::DVec4, n: f64, divide_by_n_dot_l: bool) -> Self {
        Self {
            color,
            n,
            divide_by_n_dot_l,
        }
    }
}

#[typetag::serde]
impl BSDF for Blinnphong {
    fn sample(
        &self,
        _wo: &glm::DVec3,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData> {
        // TODO: Need to figure out proper sampling for this

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

    fn eval(&self, wi: &glm::DVec3, wo: &glm::DVec3, intersect_info: &IntersectInfo) -> glm::DVec3 {
        let color = glm::vec4_to_vec3(&self.color);

        let h = (-wi + wo).normalize();

        let val = intersect_info
            .get_normal()
            .unwrap()
            .dot(&h)
            .max(0.0)
            .powf(self.n);

        let val = if self.divide_by_n_dot_l {
            val / intersect_info.get_normal().unwrap().dot(&-wi)
        } else {
            val
        };

        color.component_mul(&glm::vec3(val, val, val))
    }

    fn get_bsdf_name(&self) -> &str {
        "Blinnphong"
    }

    fn get_base_color(&self) -> glm::DVec3 {
        glm::vec4_to_vec3(&self.color)
    }
}

impl DrawUI for Blinnphong {
    fn draw_ui(&self, ui: &mut egui::Ui) {
        ui.label(format!("BSDF: {}", self.get_bsdf_name()));
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui) {
        ui::color_edit_button_dvec4(ui, "Base Color", &mut self.color);
        ui.add(egui::Slider::new(&mut self.n, 1.0..=1000.0).text("n"));
        ui.checkbox(&mut self.divide_by_n_dot_l, "Divide by N.L");
    }
}
