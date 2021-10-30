use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use super::utils::{ColorPicker, ColorPickerUiData};
use super::BSDFUiData;
use crate::glm;
use crate::path_trace::medium::Medium;
use crate::path_trace::texture_list::TextureList;
use crate::ui::DrawUI;

// TODO: add roughness parameter, right now it is purely reflective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refraction {
    color: ColorPicker,
    ior: f64,
}

impl Default for Refraction {
    fn default() -> Self {
        Self::new(glm::vec3(1.0, 1.0, 1.0), 1.5)
    }
}

impl Refraction {
    pub fn new(color: glm::DVec3, ior: f64) -> Self {
        Self {
            color: ColorPicker::Color(color),
            ior,
        }
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
        intersect_info: &IntersectInfo,
        texture_list: &TextureList,
    ) -> glm::DVec3 {
        self.color.get_color(intersect_info.get_uv(), texture_list)
    }

    fn get_bsdf_name(&self) -> &str {
        "Refraction"
    }

    fn get_base_color(&self, texture_list: &TextureList) -> glm::DVec3 {
        self.color.get_color(&glm::zero(), texture_list)
    }

    fn get_ior(&self) -> f64 {
        self.ior
    }
}

impl DrawUI for Refraction {
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
        ui.add(egui::Slider::new(&mut self.ior, 1.0..=2.0).text("ior"));
    }
}
