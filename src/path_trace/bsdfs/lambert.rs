use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use super::utils::{self, ColorPicker, ColorPickerUiData};
use super::BSDFUiData;
use crate::egui;
use crate::glm;
use crate::path_trace::medium::Mediums;
use crate::path_trace::texture_list::TextureList;
use crate::ui::DrawUI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lambert {
    color: ColorPicker,
}

impl Default for Lambert {
    fn default() -> Self {
        Self::new(glm::vec3(1.0, 1.0, 1.0))
    }
}

impl Lambert {
    pub fn new(color: glm::DVec3) -> Self {
        Self {
            color: ColorPicker::Color(color),
        }
    }
}

#[typetag::serde]
impl BSDF for Lambert {
    fn sample(
        &self,
        _wo: &glm::DVec3,
        _mediums: &mut Mediums,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData> {
        if sampling_types.contains(SamplingTypes::Diffuse) {
            Some(SampleData::new(
                utils::wi_diffuse(intersect_info.get_normal().as_ref().unwrap()),
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
        intersect_info: &IntersectInfo,
        texture_list: &TextureList,
    ) -> glm::DVec3 {
        self.color
            .get_color(intersect_info.get_uv().as_ref().unwrap(), texture_list)
    }

    fn get_bsdf_name(&self) -> &str {
        "Lambert"
    }

    fn get_base_color(&self, texture_list: &TextureList) -> Option<glm::DVec3> {
        Some(self.color.get_color(&glm::zero(), texture_list))
    }

    fn set_base_color(&mut self, color: ColorPicker) {
        self.color = color;
    }
}

impl DrawUI for Lambert {
    type ExtraData = BSDFUiData;

    fn draw_ui(&self, ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {
        ui.label(format!("BSDF: {}", self.get_bsdf_name()));
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui, extra_data: &Self::ExtraData) {
        ui.horizontal(|ui| {
            ui.label("Base Color");
            self.color.draw_ui_mut(
                ui,
                &ColorPickerUiData::new(
                    extra_data.get_texture_list().clone(),
                    extra_data.get_shader_egui_id().with("Base Color"),
                ),
            );
        });
    }
}
