use super::{
    bsdfs::{utils::ColorPicker, BSDFUiData},
    intersectable::IntersectInfo,
    medium::Mediums,
    spectrum::{DSpectrum, Wavelengths},
    texture_list::TextureList,
};
use crate::{glm, ui::DrawUI};

use enumflags2::{bitflags, BitFlags};

#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SamplingTypes {
    Diffuse,
    Glossy,
    Reflection,
}

/// Stores information about the incoming ray direction (`wi`) and the
/// type of sampling used to get `wi`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampleData {
    wi: glm::DVec3,
    sampling_type: SamplingTypes,
}

impl SampleData {
    pub fn new(wi: glm::DVec3, sampling_type: SamplingTypes) -> Self {
        Self { wi, sampling_type }
    }

    pub fn get_wi(&self) -> &glm::DVec3 {
        &self.wi
    }

    pub fn get_sampling_type(&self) -> SamplingTypes {
        self.sampling_type
    }
}

#[typetag::serde(tag = "type")]
pub trait BSDF: DrawUI<ExtraData = BSDFUiData> {
    /// Calculates `wi` given `wo` and specifies the type of sampling
    /// used.
    ///
    /// `wo`: outgoing ray direction
    ///
    /// `wi`: incoming ray direction
    ///
    /// `wavelengths`: wavelengths for which the BSDF should be
    /// sampled.
    ///
    /// `mediums`: mediums that the ray is currently in. Usually the
    /// latest medium is most useful. Depending on the material,
    /// `mediums` might be changed, add a medium or remove a
    /// medium. Take a look at [`super::bsdfs::refraction::Refraction`] for
    /// better insight.
    ///
    /// `intersect_info`: information at the point of intersection
    ///
    /// `sampling_types`: the current sampling types that are possible
    ///
    /// Need to calculate the incoming ray direction since in ray
    /// tracing, we are moving from the camera into the scene, not
    /// from the light sources towards the camera. So it is reversed,
    /// we have the outgoing ray but don't have the incoming ray.
    ///
    /// If the shader is going to sample a diffuse type of sample,
    /// `sample()` should return `SampleData` only if
    /// SamplingTypes::Diffuse is contained in `sampling_types`.
    ///
    /// TODO(ish): it might make sense to pass only one wavelength to
    /// sample() instead of all the wavelengths and have sample()
    /// decide which wavelength to use for the actual sampling, making
    /// it random may lead to very slow convergence.
    fn sample(
        &self,
        wo: &glm::DVec3,
        wavelengths: &Wavelengths,
        mediums: &mut Mediums,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData>;

    /// Calculates the colour/intensity of light that moves from `wi` towards `wo`.
    ///
    /// `wo`: outgoing ray direction
    ///
    /// `wi`: incoming ray direction
    ///
    /// `wavelengths`: wavelengths for which the BSDF should be
    /// evalulated. The spectrum generated should contain only the
    /// wavelengths provided. The most common way to do this is to use
    /// [`super::spectrum::TSpectrum::from_srgb_for_wavelengths()`].
    ///
    /// `intersect_info`: information at the point of intersection
    ///
    /// `texture_list`: texture list
    ///
    /// TODO: when different sampling type(s) are used, instead of
    /// just returning the colour/intensity of light, it will need to
    /// evaluate and update the value for each pass (diffuse, glossy,
    /// reflection).
    fn eval(
        &self,
        wi: &glm::DVec3,
        wo: &glm::DVec3,
        wavelengths: &Wavelengths,
        intersect_info: &IntersectInfo,
        texture_list: &TextureList,
    ) -> DSpectrum;

    /// Calculates the colour/intensity of light produced by the object the point of intersection
    fn emission(
        &self,
        _wo: &glm::DVec3,
        _mediums: &Mediums,
        _wavelengths: &Wavelengths,
        _intersect_info: &IntersectInfo,
        _texture_list: &TextureList,
    ) -> Option<DSpectrum> {
        None
    }

    fn get_bsdf_name(&self) -> &str;

    fn get_base_color(&self, texture_list: &TextureList) -> Option<glm::DVec3>;

    fn set_base_color(&mut self, color: ColorPicker);

    fn get_ior(&self) -> f64 {
        1.0
    }
}
