// PAT
// 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
use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use super::utils::{ColorPicker, ColorPickerUiData};
use super::BSDFUiData;
use crate::egui;
use crate::glm;
use crate::path_trace::medium::Mediums;
use crate::path_trace::texture_list::TextureList;
use crate::ui::DrawUI;

pub enum IntersectInfoType {
    T,
    Point,
    Bary_Coords,
    Primitive_index,
    ObjectId,
    ShaderId,
    UV,
    Normal,
    Front_face,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugBSDF {
    info_type: IntersectInfoType,
}

impl Default for DebugBSDF {
    fn default() -> Self {
        Self::new(glm::vec3(1.0, 1.0, 1.0), 1.0)
    }
}

impl DebugBSDF {
    pub fn new(color: glm::DVec3, power: f64) -> Self {
        Self {
            color: ColorPicker::Color(color),
            power,
        }
    }
}

#[typetag::serde]
impl BSDF for DebugBSDF {
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
        unreachable!("DebugBSDF has no eval")
    }

    fn emission(
        &self,
        intersect_info: &IntersectInfo,
        texture_list: &TextureList,
    ) -> Option<glm::DVec3> {
        match self.info_type {
            IntersectInfoType::T => Some(glm::vec3(
                intersect_info.t,
                intersect_info.t,
                intersect_info.t,
            )),
            IntersectInfoType::Point => Some(intersect_info.get_point()),
            IntersectInfoType::Bary_Coords => Some(intersect_info.get_bary_coords()),
            IntersectInfoType::Primitive_index => {
                // Add hash here
                Some(glm::vec3(
                    intersect_info.get_primitive_index().unwrap(),
                    intersect_info.get_primitive_index().unwrap(),
                    intersect_info.get_primitive_index().unwrap(),
                ))
            }
            IntersectInfoType::ObjectID => {
                // Add hash here
                let mut s = DefaultHasher::new();
                let t = intersect_info.get_object_id().unwrap();
                t.hash(&mut s);
                let hashed_val: u64 = s.finish();
                let hashed_float = hashed_val.to_f64();
                Some(intersect_info.point)
            }
            IntersectInfoType::ShaderID => {
                // Add hash here
                let mut s = DefaultHasher::new();
                let t = intersect_info.get_object_id().unwrap();
                t.hash(&mut s);
                let hashed_val: u64 = s.finish();
                let hashed_float = hashed_val.to_f64();
                Some(intersect_info.point)
            }
            IntersectInfoType::UV => Some(glm::vec2_to_vec3(intersect_info.get_uv().unwrap())),
            IntersectInfoType::Normal => Some(intersect_info.get_normal().unwrap()),
            IntersectInfoType::Front_face => {
                let fron_colour = if intersect_info.get_front_face() {
                    glm::vec3(1.0, 0.0, 0.0)
                } else {
                    None
                };
                Some(fron_colour)
            }
        }
    }

    fn get_bsdf_name(&self) -> &str {
        "DebugBSDF"
    }

    fn get_base_color(&self, texture_list: &TextureList) -> glm::DVec3 {
        self.color.get_color(&glm::zero(), texture_list)
    }

    fn set_base_color(&mut self, color: ColorPicker) {
        self.color = color;
    }
}

impl DrawUI for DebugBSDF {
    type ExtraData = BSDFUiData;

    fn draw_ui(&self, ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {
        ui.label(format!("BSDF: {}", self.get_bsdf_name()));
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui, extra_data: &Self::ExtraData) {
        ui.horizontal(|ui| {
            ui.label("Debug Shader Picker");
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
