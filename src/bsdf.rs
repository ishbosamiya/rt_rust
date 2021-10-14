use crate::glm;
use crate::intersectable::IntersectInfo;

use enumflags2::{bitflags, BitFlags};

#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SamplingTypes {
    Diffuse,
    Glossy,
    Reflection,
}

pub trait BSDF {
    /// Calculates `wi` given `wo`.
    ///
    /// `wo`: outgoing ray direction
    /// `wi`: incoming ray direction
    /// `intersect_info`: information at the point of intersection
    /// `sampling_types`: the current sampling types that are possible
    ///
    /// Need to calculate the incoming ray direction since in ray
    /// tracing, we are moving from the camera into the scene, not
    /// from the light sources towards the camera. So it is reversed,
    /// we have the outgoing ray but don't have the incoming ray.
    ///
    /// If the shader is going to sample a diffuse type of sample,
    /// `sample()` should return `wi` only if SamplingTypes::Diffuse
    /// is contained in `sampling_types`.
    fn sample(
        &self,
        wo: &glm::DVec3,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<glm::DVec3>;

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
    fn eval(&self, wi: &glm::DVec3, wo: &glm::DVec3, intersect_info: &IntersectInfo) -> glm::DVec3;
}
