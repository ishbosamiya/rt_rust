use crate::glm;
use rand::Rng;
use crate::bsdf::BSDF;
extern crate rand;
use crate::sampler;
use rand::thread_rng;

pub struct Lambert {
    pub reflectance: f64
}

impl BSDF for Lambert {
    fn new() -> Self {
        Lambert {
            reflectance: 1.0_f64
        }
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
        let a :f64 = glm::one_over_pi();
        return glm::vec3(a,a,a);
    }



    fn sample( &self, 
        outgoing : &glm::DVec3, 
        vertex : &glm::DVec3,
        
    ) -> glm::DVec3 {
        let mut rng = thread_rng();
        let x: f64 = rng.gen_range(0.0..1.0);
        let y: f64 = rng.gen_range(0.0..1.0);
        let s = glm::vec2(x, y);
        let normal = vertex.normalize();
        let mut incoming = sampler::cosine_hemisphere(&s);
        incoming = incoming[0] * outgoing + incoming[1] * normal + incoming[2] * vertex;
        let wi = sampler::cosine_hemisphere(&s);

        
        let prob : f64 = wi[1] * 0.31830988_f64 ;

        assert!(prob > 0.0_f64);
        incoming
    }
}
