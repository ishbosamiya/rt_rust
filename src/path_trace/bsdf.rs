use super::{
    bsdfs::{utils::ColorPicker, BSDFUiData},
    intersectable::IntersectInfo,
    medium::Mediums,
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
    fn sample(
        &self,
        wo: &glm::DVec3,
        mediums: &mut Mediums,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData>;

    /// Calculates the colour/intensity of light that moves from `wi` towards `wo`.
    ///
    /// `wo`: outgoing ray direction
    /// `wi`: incoming ray direction
    /// `intersect_info`: information at the point of intersection
    ///
    /// TODO: when different sampling type(s) are used, instead of
    /// just returning the colour/intensity of light, it will need to
    /// evaluate and update the value for each pass (diffuse, glossy,
    /// reflection).
    fn eval(
        &self,
        wi: &glm::DVec3,
        wo: &glm::DVec3,
        intersect_info: &IntersectInfo,
        texture_list: &TextureList,
    ) -> glm::DVec3;

    /// Calculates the colour/intensity of light produced by the object the point of intersection
    fn emission(
        &self,
        _wo: &glm::DVec3,
        _mediums: &Mediums,
        _intersect_info: &IntersectInfo,
        _texture_list: &TextureList,
    ) -> Option<glm::DVec3> {
        None
    }

    fn get_bsdf_name(&self) -> &str;

    fn get_base_color(&self, texture_list: &TextureList) -> Option<glm::DVec3>;

    fn set_base_color(&mut self, color: ColorPicker);

    fn get_ior(&self) -> f64 {
        1.0
    }
}
