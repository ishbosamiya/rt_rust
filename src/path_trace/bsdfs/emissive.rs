use enumflags2::BitFlags;

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use crate::glm;

pub struct Emissive {
    color: glm::DVec4,
    power: f64,
}

impl Emissive {
    pub fn new(color: glm::DVec4, power: f64) -> Self {
        Self { color, power }
    }
}

impl BSDF for Emissive {
    fn sample(
        &self,
        _wo: &glm::DVec3,
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
    ) -> glm::DVec3 {
        unreachable!("Emissive only material, so no eval is possible")
    }

    fn emission(&self, _intersect_info: &IntersectInfo) -> Option<glm::DVec3> {
        Some(glm::vec4_to_vec3(&(self.power * self.color)))
    }
}
