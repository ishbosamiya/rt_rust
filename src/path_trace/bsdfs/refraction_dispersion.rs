use std::fmt::Display;

use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use super::utils::{self, ColorPicker, ColorPickerUiData};
use super::BSDFUiData;
use crate::egui;
use crate::glm;
use crate::path_trace::medium::{Medium, Mediums};
use crate::path_trace::spectrum::{DSpectrum, TSpectrum, Wavelengths};
use crate::path_trace::texture_list::TextureList;
use crate::ui::DrawUI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Material {
    Diamond,
}

impl Display for Material {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Material::Diamond => write!(f, "Diamond"),
        }
    }
}

impl Material {
    pub fn all() -> impl Iterator<Item = Self> {
        use Material::*;
        [Diamond].iter().copied()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefractionDispersion {
    color: ColorPicker,

    material: Material,

    #[serde(default = "default_roughness")]
    roughness: f64,
}

fn default_roughness() -> f64 {
    // any previous files assumed roughness of 0.0
    0.0
}

impl Default for RefractionDispersion {
    fn default() -> Self {
        Self::new(glm::vec3(1.0, 1.0, 1.0), Material::Diamond, 0.0)
    }
}

impl RefractionDispersion {
    pub fn new(color: glm::DVec3, material: Material, roughness: f64) -> Self {
        Self {
            color: ColorPicker::Color(color),
            material,
            roughness,
        }
    }

    /// Calculate ior of the material at the given wavelength using
    /// Sellmeir's equation.
    fn calculate_ior(&self, wavelength: usize) -> f64 {
        match self.material {
            Material::Diamond => {
                // reference:
                // https://refractiveindex.info/?shelf=3d&book=crystals&page=diamond
                let iors = vec![
                    (365, 2.473323675),
                    (387, 2.464986815),
                    (413, 2.455051934),
                    (443, 2.441251728),
                    (477, 2.431478974),
                    (517, 2.427076431),
                    (564, 2.420857286),
                    (620, 2.411429037),
                    (689, 2.406543164),
                    (775, 2.406202402),
                    (886, 2.400035416),
                ];

                let mut ior = 0.0;
                for (i, (known_wavelength, known_ior)) in iors.iter().enumerate() {
                    match wavelength.cmp(known_wavelength) {
                        std::cmp::Ordering::Less => {}
                        std::cmp::Ordering::Equal => {
                            ior = *known_ior;
                            break;
                        }
                        std::cmp::Ordering::Greater => {
                            let (higher_wavelength, higher_refractive_index) = iors[i + 1];
                            let (lower_wavelength, lower_refractive_index): (usize, f64) =
                                (*known_wavelength, *known_ior);

                            ior = glm::lerp_scalar(
                                lower_refractive_index,
                                higher_refractive_index,
                                (wavelength - lower_wavelength) as f64
                                    / (higher_wavelength - lower_wavelength) as f64,
                            );
                            break;
                        }
                    }
                }
                assert!(ior != 0.0);
                ior
            }
        }
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

impl DrawUI for Material {
    type ExtraData = BSDFUiData;

    fn draw_ui(&self, _ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {
        unreachable!("no non mut draw ui for IntersectInfoType")
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui, extra_data: &Self::ExtraData) {
        egui::ComboBox::from_id_source(extra_data.get_shader_egui_id().with("Material"))
            .selected_text(format!("{}", self))
            .show_ui(ui, |ui| {
                Self::all().for_each(|info| {
                    ui.selectable_value(self, info, format!("{}", info));
                });
            });
    }
}
