use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use super::utils::ColorPicker;
use super::BSDFUiData;
use crate::egui;
use crate::glm;
use crate::path_trace::medium::Mediums;
use crate::path_trace::spectrum::{DSpectrum, TSpectrum, Wavelengths};
use crate::path_trace::texture_list::TextureList;
use crate::ui::DrawUI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntersectInfoType {
    T,
    Point,
    BaryCoords,
    PrimitiveIndex,
    ObjectID,
    ShaderID,
    UV,
    Normal,
    FrontFace,
}

impl Display for IntersectInfoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntersectInfoType::T => write!(f, "Ray Distance"),
            IntersectInfoType::Point => write!(f, "Point"),
            IntersectInfoType::BaryCoords => write!(f, "Barycentric Coords"),
            IntersectInfoType::PrimitiveIndex => write!(f, "Primitive Index"),
            IntersectInfoType::ObjectID => write!(f, "Object ID"),
            IntersectInfoType::ShaderID => write!(f, "Shader ID"),
            IntersectInfoType::UV => write!(f, "UV"),
            IntersectInfoType::Normal => write!(f, "Normal"),
            IntersectInfoType::FrontFace => write!(f, "Front Face"),
        }
    }
}

impl IntersectInfoType {
    pub fn all() -> impl Iterator<Item = Self> {
        use IntersectInfoType::*;
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
    info_type: IntersectInfoType,
    /// Divide the intersect info t with this distance factor
    distance_factor: f64,
}

impl Default for DebugBSDF {
    fn default() -> Self {
        Self::new(IntersectInfoType::T, 25.0)
    }
}

impl DebugBSDF {
    pub fn new(info_type: IntersectInfoType, distance_factor: f64) -> Self {
        Self {
            info_type,
            distance_factor,
        }
    }

    fn get_color(&self, intersect_info: &IntersectInfo) -> glm::DVec3 {
        match self.info_type {
            IntersectInfoType::T => {
                let val = intersect_info.get_t() / self.distance_factor;
                glm::vec3(val, val, val)
            }
            IntersectInfoType::Point => *intersect_info.get_point(),
            IntersectInfoType::BaryCoords => *intersect_info.get_bary_coords(),
            IntersectInfoType::PrimitiveIndex => hash_to_rgb(intersect_info.get_primitive_index()),
            IntersectInfoType::ObjectID => hash_to_rgb(&intersect_info.get_object_id()),
            IntersectInfoType::ShaderID => hash_to_rgb(&intersect_info.get_shader_id()),
            IntersectInfoType::UV => glm::vec2_to_vec3(intersect_info.get_uv().as_ref().unwrap()),
            IntersectInfoType::Normal => intersect_info.get_normal().unwrap(),
            IntersectInfoType::FrontFace => {
                if intersect_info.get_front_face() {
                    glm::vec3(0.0, 0.0, 1.0)
                } else {
                    glm::vec3(1.0, 0.0, 0.0)
                }
            }
        }
    }
}

#[typetag::serde]
impl BSDF for DebugBSDF {
    fn sample(
        &self,
        _wo: &glm::DVec3,
        _wavelengths: &Wavelengths,
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
        _wavelengths: &Wavelengths,
        _intersect_info: &IntersectInfo,
        _texture_list: &TextureList,
    ) -> DSpectrum {
        unreachable!("DebugBSDF has no eval")
    }

    fn emission(
        &self,
        wavelengths: &Wavelengths,
        intersect_info: &IntersectInfo,
        _texture_list: &TextureList,
    ) -> Option<DSpectrum> {
        Some(TSpectrum::from_srgb_for_wavelengths(
            &self.get_color(intersect_info),
            wavelengths,
        ))
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

        if self.info_type == IntersectInfoType::T {
            ui.add(
                egui::Slider::new(&mut self.distance_factor, 0.00001..=25.0)
                    .text("Distance Factor"),
            );
        }
    }
}

impl DrawUI for IntersectInfoType {
    type ExtraData = BSDFUiData;

    fn draw_ui(&self, _ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {
        unreachable!("no non mut draw ui for IntersectInfoType")
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui, extra_data: &Self::ExtraData) {
        egui::ComboBox::from_id_source(extra_data.get_shader_egui_id().with("IntersectInfoType"))
            .selected_text(format!("{}", self))
            .show_ui(ui, |ui| {
                Self::all().for_each(|info| {
                    ui.selectable_value(self, info, format!("{}", info));
                });
            });
    }
}
