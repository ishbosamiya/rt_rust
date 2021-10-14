use enumflags2::BitFlags;

use crate::bsdf::{SamplingTypes, BSDF};
use crate::glm;
use crate::intersectable::IntersectInfo;

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
    ) -> Option<glm::DVec3> {
        if sampling_types.contains(SamplingTypes::Reflection) {
            Some(glm::reflect_vec(
                wo,
                intersect_info.get_normal().as_ref().unwrap(),
            ))
        } else {
            None
        }
    }

    fn eval(
        &self,
        _wi: &glm::DVec3,
        _wo: &glm::DVec3,
        _intersect_info: &crate::intersectable::IntersectInfo,
    ) -> glm::DVec3 {
        #[allow(clippy::let_and_return)]
        let color = glm::vec4_to_vec3(&self.color);

        color
    }
}
