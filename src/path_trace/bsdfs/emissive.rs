use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use super::utils::{ColorPicker, ColorPickerUiData};
use super::BSDFUiData;
use crate::glm;
use crate::path_trace::medium::Mediums;
use crate::path_trace::texture_list::TextureList;
use crate::ui::DrawUI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emissive {
    color: ColorPicker,
    power: f64,
}

impl Default for Emissive {
    fn default() -> Self {
        Self::new(glm::vec3(1.0, 1.0, 1.0), 1.0)
    }
}

impl Emissive {
    pub fn new(color: glm::DVec3, power: f64) -> Self {
        Self {
            color: ColorPicker::Color(color),
            power,
        }
    }
}

#[typetag::serde]
impl BSDF for Emissive {
    fn sample(
        &self,
        _wo: &glm::DVec3,
        _mediums: &mut Mediums,
        _intersect_info: &IntersectInfo,
        _sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData> {
        None
    }

    fn eval(
        &self,
        _wi: &glm::DVec3,
        _wo: &glm::DVec3,
        _intersect_info: &IntersectInfo,
        _texture_list: &TextureList,
    ) -> glm::DVec3 {
        unreachable!("Emissive only material, so no eval is possible")
    }

    fn emission(
        &self,
        intersect_info: &IntersectInfo,
        texture_list: &TextureList,
    ) -> Option<glm::DVec3> {
        Some(self.power * self.color.get_color(intersect_info.get_uv(), texture_list))
    }

    fn get_bsdf_name(&self) -> &str {
        "Emissive"
    }

    fn get_base_color(&self, texture_list: &TextureList) -> glm::DVec3 {
        self.color.get_color(&glm::zero(), texture_list)
    }
}

impl DrawUI for Emissive {
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
        ui.add(egui::Slider::new(&mut self.power, 0.0..=10.0).text("Power"));
    }
}
