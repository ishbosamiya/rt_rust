use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use super::utils::{self, ColorPicker, ColorPickerUiData, DispersiveMaterial};
use super::BSDFUiData;
use crate::egui;
use crate::glm;
use crate::path_trace::medium::{Medium, Mediums};
use crate::path_trace::spectrum::{DSpectrum, TSpectrum, Wavelengths};
use crate::path_trace::texture_list::TextureList;
use crate::ui::DrawUI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefractionDispersion {
    color: ColorPicker,

    material: DispersiveMaterial,

    #[serde(default = "default_roughness")]
    roughness: f64,
}

fn default_roughness() -> f64 {
    // any previous files assumed roughness of 0.0
    0.0
}

impl Default for RefractionDispersion {
    fn default() -> Self {
        Self::new(glm::vec3(1.0, 1.0, 1.0), DispersiveMaterial::Diamond, 0.0)
    }
}

impl RefractionDispersion {
    pub fn new(color: glm::DVec3, material: DispersiveMaterial, roughness: f64) -> Self {
        Self {
            color: ColorPicker::Color(color),
            material,
            roughness,
        }
    }

    fn calculate_ior(&self, wavelength: usize) -> f64 {
        self.material.calculate_ior(wavelength)
    }

    fn handle_refraction(
        &self,
        wo: &glm::DVec3,
        wavelength: usize,
        mediums: &mut Mediums,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData> {
        // calculate ior based on the given wavelength
        let self_ior = self.calculate_ior(wavelength);

        // TODO: need to figure out which sampling type it would be,
        // both diffuse and reflection seem to make sense
        if sampling_types.contains(SamplingTypes::Diffuse) {
            let entering = intersect_info.get_front_face();

            let ior = if entering {
                let ior = mediums.get_lastest_medium().unwrap().get_ior() / self_ior;
                ior
            } else {
                // need to remove the lastest medium since it would be
                // the same as the medium of wi
                mediums.remove_medium().unwrap();

                // if there is no more mediums, the ray must have had
                // more exits than entries and thus must not
                // sample. This can happen because of non manifold
                // meshes. If there exists a medium, calculate the
                // ior.
                self_ior / mediums.get_lastest_medium()?.get_ior()
            };

            let output =
                -glm::refract_vec(&-wo, intersect_info.get_normal().as_ref().unwrap(), ior);

            // if refraction can take place. It may not be possible
            // when `wi` is at an angle (with the normal) greater than
            // critical angle and `wi` would be in a denser medium
            // than `wo`. In such a case total internal reflection
            // will take place but in refraction only bsdf, this isn't
            // considered
            if output != glm::DVec3::zeros() {
                // add `wi` medium if entering the medium
                if entering {
                    mediums.add_medium(Medium::new(self_ior));
                }

                Some(SampleData::new(output, SamplingTypes::Diffuse))
            } else {
                None
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
impl BSDF for RefractionDispersion {
    fn sample(
        &self,
        wo: &glm::DVec3,
        wavelengths: &Wavelengths,
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
            // sample pure refraction
            self.handle_refraction(
                wo,
                wavelengths.get_wavelengths()[(wavelengths.get_wavelengths().len() / 2)],
                mediums,
                intersect_info,
                sampling_types,
            )
        }
    }

    fn eval(
        &self,
        _wi: &glm::DVec3,
        _wo: &glm::DVec3,
        wavelengths: &Wavelengths,
        intersect_info: &IntersectInfo,
        texture_list: &TextureList,
    ) -> DSpectrum {
        TSpectrum::from_srgb_for_wavelengths(
            &self
                .color
                .get_color(intersect_info.get_uv().as_ref().unwrap(), texture_list),
            wavelengths,
        )
    }

    fn get_bsdf_name(&self) -> &str {
        "RefractionDispersion"
    }

    fn get_base_color(&self, texture_list: &TextureList) -> Option<glm::DVec3> {
        Some(self.color.get_color(&glm::zero(), texture_list))
    }

    fn set_base_color(&mut self, color: ColorPicker) {
        self.color = color;
    }

    fn get_ior(&self) -> f64 {
        self.calculate_ior(580)
    }
}

impl DrawUI for RefractionDispersion {
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

        self.material.draw_ui_mut(ui, extra_data);
        ui.add(egui::Slider::new(&mut self.roughness, 0.0..=1.0).text("Roughness"));
    }
}
