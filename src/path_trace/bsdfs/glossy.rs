use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use super::utils::{self, ColorPicker, ColorPickerUiData};
use super::BSDFUiData;
use crate::glm;
use crate::path_trace::medium::Mediums;
use crate::path_trace::texture_list::TextureList;
use crate::ui::DrawUI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glossy {
    color: ColorPicker,

    #[serde(default = "default_roughness")]
    roughness: f64,
}

fn default_roughness() -> f64 {
    // any previous files assumed roughness of 0.0
    0.0
}

impl Default for Glossy {
    fn default() -> Self {
        Self::new(glm::vec3(1.0, 1.0, 1.0), 0.4)
    }
}

impl Glossy {
    pub fn new(color: glm::DVec3, roughness: f64) -> Self {
        Self {
            color: ColorPicker::Color(color),
            roughness,
        }
    }

    fn handle_reflection(
        &self,
        wo: &glm::DVec3,
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

    fn handle_diffuse(
        &self,
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
}

#[typetag::serde]
impl BSDF for Glossy {
    fn sample(
        &self,
        wo: &glm::DVec3,
        _mediums: &mut Mediums,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData> {
        // TODO(ish): need to handle roughness accurately, using
        // something like GGX microfacet model. Right now it cannot
        // give physically accurate results.
        if rand::random::<f64>() < self.roughness {
            // sample diffuse
            self.handle_diffuse(intersect_info, sampling_types)
        } else {
            // sample pure reflection
            self.handle_reflection(wo, intersect_info, sampling_types)
        }
    }

    fn eval(
        &self,
        _wi: &glm::DVec3,
        _wo: &glm::DVec3,
        intersect_info: &IntersectInfo,
        texture_list: &TextureList,
    ) -> glm::DVec3 {
        self.color.get_color(intersect_info.get_uv(), texture_list)
    }

    fn get_bsdf_name(&self) -> &str {
        "Glossy"
    }

    fn get_base_color(&self, texture_list: &TextureList) -> glm::DVec3 {
        self.color.get_color(&glm::zero(), texture_list)
    }
}

impl DrawUI for Glossy {
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
        ui.add(egui::Slider::new(&mut self.roughness, 0.0..=1.0).text("Roughness"));
    }
}
