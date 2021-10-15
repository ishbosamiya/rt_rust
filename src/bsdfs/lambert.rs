use enumflags2::BitFlags;

use crate::bsdf::{SampleData, SamplingTypes, BSDF};
use crate::glm;
use crate::intersectable::IntersectInfo;
use crate::math;

pub struct Lambert {
    color: glm::DVec4,
}

impl Lambert {
    pub fn new(color: glm::DVec4) -> Self {
        Self { color }
    }
}

impl BSDF for Lambert {
    fn sample(
        &self,
        _wo: &glm::DVec3,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData> {
        // TODO: make this random in hemisphere instead of using a
        // sphere for better performance
        if sampling_types.contains(SamplingTypes::Diffuse) {
            Some(SampleData::new(
                intersect_info.get_normal().unwrap() + math::random_in_unit_sphere(),
                SamplingTypes::Diffuse,
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
