use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use super::utils::{ColorPicker, ColorPickerUiData};
use super::BSDFUiData;
use crate::glm;
use crate::math;
use crate::path_trace::medium::Mediums;
use crate::path_trace::texture_list::TextureList;
use crate::ui::DrawUI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blinnphong {
    color: ColorPicker,
    n: f64,
    divide_by_n_dot_l: bool,
}

impl Default for Blinnphong {
    fn default() -> Self {
        Self::new(glm::vec3(1.0, 1.0, 1.0), 100.0, false)
    }
}

impl Blinnphong {
    pub fn new(color: glm::DVec3, n: f64, divide_by_n_dot_l: bool) -> Self {
        Self {
            color: ColorPicker::Color(color),
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
        _mediums: &mut Mediums,
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

    fn eval(
        &self,
        wi: &glm::DVec3,
        wo: &glm::DVec3,
        intersect_info: &IntersectInfo,
        texture_list: &TextureList,
    ) -> glm::DVec3 {
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

        self.color
            .get_color(intersect_info.get_uv(), texture_list)
            .component_mul(&glm::vec3(val, val, val))
    }

    fn get_bsdf_name(&self) -> &str {
        "Blinnphong"
    }

    fn get_base_color(&self, texture_list: &TextureList) -> glm::DVec3 {
        self.color.get_color(&glm::zero(), texture_list)
    }

    fn set_base_color(&mut self, color: ColorPicker) {
        self.color = color;
    }
}

impl DrawUI for Blinnphong {
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
        ui.add(egui::Slider::new(&mut self.n, 1.0..=1000.0).text("n"));
        ui.checkbox(&mut self.divide_by_n_dot_l, "Divide by N.L");
    }
}
