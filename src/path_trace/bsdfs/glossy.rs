use enumflags2::BitFlags;

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use crate::glm;

// TODO: add roughness parameter, right now it is purely reflective
pub struct Glossy {
    color: glm::DVec4,
}

impl Glossy {
    pub fn new(color: glm::DVec4) -> Self {
        Self { color }
    }
}

impl BSDF for Glossy {
    fn sample(
        &self,
        wo: &glm::DVec3,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData> {
        if sampling_types.contains(SamplingTypes::Reflection) {
            // need to consider the inverse of the outgoing direction
            // during reflection
            Some(SampleData::new(
                glm::reflect_vec(&-wo, intersect_info.get_normal().as_ref().unwrap()),
                SamplingTypes::Reflection,
            ))
        } else {
            None
        }
    }

    fn eval(
        &self,
        _wi: &glm::DVec3,
        _wo: &glm::DVec3,
        _intersect_info: &IntersectInfo,
    ) -> glm::DVec3 {
        #[allow(clippy::let_and_return)]
        let color = glm::vec4_to_vec3(&self.color);

        color
    }
}
