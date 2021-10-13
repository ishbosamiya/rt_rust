use crate::bsdf::BSDF;
use crate::glm;
use crate::intersectable::IntersectInfo;
use rand::Rng;
extern crate rand;
use crate::sampler;
use rand::thread_rng;

pub struct Lambert {
    pub reflectance: f64,
}

impl BSDF for Lambert {
    fn new() -> Self {
        Lambert {
            reflectance: 1.0_f64,
        }
    }

    fn sample(&self, outgoing: &glm::DVec3, intersect_info: &IntersectInfo) -> glm::DVec3 {
        let mut rng = thread_rng();
        let x: f64 = rng.gen_range(0.0..1.0);
        let y: f64 = rng.gen_range(0.0..1.0);
        let s = glm::vec2(x, y);
        let normal = intersect_info.get_normal().unwrap();
        let incoming = sampler::cosine_hemisphere(&s);
        let incoming = incoming[0] * outgoing
            + incoming[1] * normal
            + incoming[2] * intersect_info.get_point();
        let wi = sampler::cosine_hemisphere(&s);
        let prob: f64 = wi[1] * std::f64::consts::FRAC_1_PI;
        assert!(prob > 0.0);
        incoming
    }

    fn eval(
        &self,
        _l: &glm::DVec3,
        _v: &glm::DVec3,
        _n: &glm::DVec3,
        _x: &glm::DVec3,
        _y: &glm::DVec3,
    ) -> glm::DVec3 {
        // Simple function according to disney brdf
        let a: f64 = glm::one_over_pi();
        glm::vec3(a, a, a)
    }
}
