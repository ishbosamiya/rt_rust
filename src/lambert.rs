use crate::glm;
use crate::bsdf::BSDF;

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
        let val: f64 = self.reflectance / std::f64::consts::PI;
        glm::vec3(val, val, val)
    }
    fn sample(&self, 
        _out : &glm::DVec3, 
        _vertex : &glm::DVec3
    ) -> glm::DVec3 {
        glm::zero()
    }
}
