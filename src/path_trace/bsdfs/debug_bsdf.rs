use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use super::utils::{self, ColorPicker};
use super::BSDFUiData;
use crate::egui;
use crate::glm;
use crate::path_trace::medium::Mediums;
use crate::path_trace::texture_list::TextureList;
use crate::ui::DrawUI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InfoType {
    T,
    Point,
    BaryCoords,
    PrimitiveIndex,
    ObjectID,
    ShaderID,
    UV,
    Normal,
    FrontFace,
    Fresnel,
}

impl Display for InfoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InfoType::T => write!(f, "Ray Distance"),
            InfoType::Point => write!(f, "Point"),
            InfoType::BaryCoords => write!(f, "Barycentric Coords"),
            InfoType::PrimitiveIndex => write!(f, "Primitive Index"),
            InfoType::ObjectID => write!(f, "Object ID"),
            InfoType::ShaderID => write!(f, "Shader ID"),
            InfoType::UV => write!(f, "UV"),
            InfoType::Normal => write!(f, "Normal"),
            InfoType::FrontFace => write!(f, "Front Face"),
            InfoType::Fresnel => write!(f, "Fresnel"),
        }
    }
}

impl InfoType {
    pub fn all() -> impl Iterator<Item = Self> {
        use InfoType::*;
        [
            T,
            Point,
            BaryCoords,
            PrimitiveIndex,
            ObjectID,
            ShaderID,
            UV,
            Normal,
            FrontFace,
            Fresnel,
        ]
        .iter()
        .copied()
    }
}

fn hash_to_rgb(val: &impl Hash) -> glm::DVec3 {
    let mut s = DefaultHasher::new();
    val.hash(&mut s);
    glm::convert(glm::make_vec3(&egui::color::rgb_from_hsv((
        s.finish() as f32 / u64::MAX as f32,
        0.9,
        1.0,
    ))))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugBSDF {
    info_type: InfoType,
    /// Divide the intersect info t with this distance factor
    distance_factor: f64,
    /// Index of refraction of the material
    ior: f64,
}

impl Default for DebugBSDF {
    fn default() -> Self {
        Self::new(InfoType::T, 25.0, 1.5)
    }
}

impl DebugBSDF {
    pub fn new(info_type: InfoType, distance_factor: f64, ior: f64) -> Self {
        Self {
            info_type,
            distance_factor,
            ior,
        }
    }

    fn get_color(
        &self,
        wo: &glm::DVec3,
        mediums: &Mediums,
        intersect_info: &IntersectInfo,
    ) -> glm::DVec3 {
        match self.info_type {
            InfoType::T => {
                let val = intersect_info.get_t() / self.distance_factor;
                glm::vec3(val, val, val)
            }
            InfoType::Point => *intersect_info.get_point(),
            InfoType::BaryCoords => *intersect_info.get_bary_coords(),
            InfoType::PrimitiveIndex => hash_to_rgb(intersect_info.get_primitive_index()),
            InfoType::ObjectID => hash_to_rgb(&intersect_info.get_object_id()),
            InfoType::ShaderID => hash_to_rgb(&intersect_info.get_shader_id()),
            InfoType::UV => glm::vec2_to_vec3(intersect_info.get_uv().as_ref().unwrap()),
            InfoType::Normal => intersect_info.get_normal().unwrap(),
            InfoType::FrontFace => {
                if intersect_info.get_front_face() {
                    glm::vec3(0.0, 0.0, 1.0)
                } else {
                    glm::vec3(1.0, 0.0, 0.0)
                }
            }
            InfoType::Fresnel => {
                let entering = intersect_info.get_front_face();
                let (n1, n2) = if entering {
                    (
                        self.get_ior(),
                        mediums.get_lastest_medium().unwrap().get_ior(),
                    )
                } else {
                    (
                        mediums.get_lastest_medium().unwrap().get_ior(),
                        self.get_ior(),
                    )
                };
                let val = utils::fresnel(intersect_info.get_normal().as_ref().unwrap(), wo, n1, n2);
                glm::vec3(val, val, val)
            }
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
        wo: &glm::DVec3,
        mediums: &Mediums,
        intersect_info: &IntersectInfo,
        _texture_list: &TextureList,
    ) -> Option<glm::DVec3> {
        Some(self.get_color(wo, mediums, intersect_info))
    }

    fn get_bsdf_name(&self) -> &str {
        "Debug BSDF"
    }

    fn get_base_color(&self, _texture_list: &TextureList) -> Option<glm::DVec3> {
        None
    }

    fn set_base_color(&mut self, _color: ColorPicker) {
        // no color to set
    }

    fn get_ior(&self) -> f64 {
        self.ior
    }
}

impl DrawUI for DebugBSDF {
    type ExtraData = BSDFUiData;

    fn draw_ui(&self, ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {
        ui.label(format!("BSDF: {}", self.get_bsdf_name()));
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui, extra_data: &Self::ExtraData) {
        ui.horizontal(|ui| {
            ui.label("Info Type: ");
            self.info_type.draw_ui_mut(ui, extra_data);
        });

        match self.info_type {
            InfoType::T => {
                ui.add(
                    egui::Slider::new(&mut self.distance_factor, 0.00001..=25.0)
                        .text("Distance Factor"),
                );
            }
            InfoType::Fresnel => {
                ui.add(egui::Slider::new(&mut self.ior, 0.0..=3.0).text("ior"));
            }
            _ => {}
        }
    }
}

impl DrawUI for InfoType {
    type ExtraData = BSDFUiData;

    fn draw_ui(&self, _ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {
        unreachable!("no non mut draw ui for InfoType")
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui, extra_data: &Self::ExtraData) {
        egui::ComboBox::from_id_source(extra_data.get_shader_egui_id().with("InfoType"))
            .selected_text(format!("{}", self))
            .show_ui(ui, |ui| {
                Self::all().for_each(|info| {
                    ui.selectable_value(self, info, format!("{}", info));
                });
            });
    }
}
