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
        l: &glm::DVec3,
        v: &glm::DVec3,
        n: &glm::DVec3,
        x: &glm::DVec3,
        y: &glm::DVec3,
    ) -> glm::DVec3 {
        // Simple function according to disney brdf
        let val = self.reflectance / 3.14159265;
        return glm::vec3(val, val, val);
    }
    /*
    fn sample(&self, out : &glm::DVec3, vertex : &glm::DVec3, bsdf_sample: &Sample) -> glm::DVec3{

    }
    */
}
