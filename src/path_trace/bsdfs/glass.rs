use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use super::utils::{self, ColorPicker, ColorPickerUiData};
use super::BSDFUiData;
use crate::glm;
use crate::path_trace::medium::{Medium, Mediums};
use crate::path_trace::texture_list::TextureList;
use crate::ui::DrawUI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glass {
    color: ColorPicker,
    ior: f64,
    roughness: f64,
}

impl Default for Glass {
    fn default() -> Self {
        Self::new(glm::vec3(1.0, 1.0, 1.0), 1.5, 0.4)
    }
}

impl Glass {
    pub fn new(color: glm::DVec3, ior: f64, roughness: f64) -> Self {
        Self {
            color: ColorPicker::Color(color),
            ior,
            roughness,
        }
    }

    fn handle_refraction_and_reflection(
        &self,
        wo: &glm::DVec3,
        mediums: &mut Mediums,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData> {
        // TODO: need to figure out which sampling type it would be,
        // both diffuse and reflection seem to make sense
        if sampling_types.contains(SamplingTypes::Diffuse | SamplingTypes::Reflection) {
            let entering = intersect_info.get_front_face();

            let medium;
            let ior = if entering {
                let ior = mediums.get_lastest_medium().unwrap().get_ior() / self.get_ior();
                medium = None;
                ior
            } else {
                // need to remove the lastest medium since it would be
                // the same as the medium of wi
                medium = Some(mediums.remove_medium().unwrap());

                // if there is no more mediums, the ray must have had
                // more exits than entries and thus must not
                // sample. This can happen because of non manifold
                // meshes. If there exists a medium, calculate the
                // ior.
                self.get_ior() / mediums.get_lastest_medium()?.get_ior()
            };

            let refracted_wi =
                -glm::refract_vec(&-wo, intersect_info.get_normal().as_ref().unwrap(), ior);

            // if refraction can take place. It may not be possible
            // when `wi` is at an angle (with the normal) greater than
            // critical angle and `wi` would be in a denser medium
            // than `wo`. In such a case total internal reflection
            // will take place.
            if refracted_wi != glm::DVec3::zeros() {
                if sampling_types.contains(SamplingTypes::Diffuse) {
                    // add `wi` medium if entering the medium
                    if entering {
                        mediums.add_medium(Medium::new(self.get_ior()));
                    }

                    Some(SampleData::new(refracted_wi, SamplingTypes::Diffuse))
                } else {
                    None
                }
            } else {
                // total internal reflection (TIR)

                // Add back the medium
                mediums.add_medium(medium.unwrap());

                if sampling_types.contains(SamplingTypes::Reflection) {
                    Some(SampleData::new(
                        glm::reflect_vec(wo, intersect_info.get_normal().as_ref().unwrap()),
                        SamplingTypes::Reflection,
                    ))
                } else {
                    None
                }
            }
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
impl BSDF for Glass {
    fn sample(
        &self,
        wo: &glm::DVec3,
        mediums: &mut Mediums,
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
            // sample refraction and reflection
            self.handle_refraction_and_reflection(wo, mediums, intersect_info, sampling_types)
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
        "Glass"
    }

    fn get_base_color(&self, texture_list: &TextureList) -> glm::DVec3 {
        self.color.get_color(&glm::zero(), texture_list)
    }

    fn get_ior(&self) -> f64 {
        self.ior
    }
}

impl DrawUI for Glass {
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
        ui.add(egui::Slider::new(&mut self.roughness, 0.0..=1.0).text("Roughness"));
    }
}
