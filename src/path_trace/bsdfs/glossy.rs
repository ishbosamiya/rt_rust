use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use super::BSDFUiData;
use crate::path_trace::medium::Medium;
use crate::path_trace::texture_list::TextureList;
use crate::ui::DrawUI;
use crate::{glm, ui};

// TODO: add roughness parameter, right now it is purely reflective
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Glossy {
    color: glm::DVec3,
}

impl Default for Glossy {
    fn default() -> Self {
        Self::new(glm::vec3(1.0, 1.0, 1.0))
    }
}

impl Glossy {
    pub fn new(color: glm::DVec3) -> Self {
        Self { color }
    }
}

#[typetag::serde]
impl BSDF for Glossy {
    fn sample(
        &self,
        wo: &glm::DVec3,
        _wo_medium: &Medium,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData> {
        if sampling_types.contains(SamplingTypes::Reflection) {
            Some(SampleData::new(
                glm::reflect_vec(wo, intersect_info.get_normal().as_ref().unwrap()),
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
        _wo_medium: &Medium,
        _intersect_info: &IntersectInfo,
        _texture_list: &TextureList,
    ) -> glm::DVec3 {
        self.color
    }

    fn get_bsdf_name(&self) -> &str {
        "Glossy"
    }

    fn get_base_color(&self, _texture_list: &TextureList) -> glm::DVec3 {
        self.color
    }
}

impl DrawUI for Glossy {
    type ExtraData = BSDFUiData;

    fn draw_ui(&self, ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {
        ui.label(format!("BSDF: {}", self.get_bsdf_name()));
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {
        ui::color_edit_button_dvec3(ui, "Base Color", &mut self.color);
    }
}
