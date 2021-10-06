use crate::bsdf::BSDF;
use crate::glm;
use crate::bsdfutils::Utils;

// File for main disney brdf code
struct Disney {
    pub metallic: f64,
    pub subsurface: f64,
    pub specular: f64,
    pub roughness: f64,
    pub specularTint: f64,
    pub anisotropic: f64,
    pub sheen: f64,
    pub sheenTint: f64,
    pub clearcoat: f64,
    pub clearcoatGloss: f64
}


impl BSDF for Disney {
    fn new() -> Self {
        Disney {metallic: 0.0_f64, 
            subsurface: 0.0_f64, 
            specular: 0.5_f64, 
            roughness: 0.5_f64, 
            specularTint: 0.0_f64, 
            anisotropic: 0.0_f64, 
            sheen: 0.0_f64, 
            sheenTint: 0.5_f64,
            clearcoat: 0.0_f64,
            clearcoatGloss: 1.0_f64
        }
    }
    
    fn eval(&self,
        l: &glm::DVec3,
        v: &glm::DVec3,
        n: &glm::DVec3,
        x: &glm::DVec3,
        y: &glm::DVec3,
    ) -> glm::DVec3 {
        // Enter main eval code
        return glm::vec3(0.0, 0.0, 0.0);
    }
}