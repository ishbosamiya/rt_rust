use crate::bsdf::BSDF;
use crate::glm;
use crate::bsdfutils::Utils;

// File for main disney brdf code
struct Disney {
    pub basecolor: glm::DVec3,
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
        Disney {basecolor: glm::DVec3::new(0.82, 0.67, 0.16),
            metallic: 0.0_f64, 
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
        let ndot_l = n.dot(l);
        let ndot_v = n.dot(v);

        if ndot_l < 0.0 || ndot_v < 0.0 {
            return glm::DVec3::new(0.0, 0.0, 0.0);
        }
        //let mut h : glm::DVec3;
        let mut h = (l + v).normalize();

        let ndot_h = n.dot(&h);
        let ldot_h = l.dot(&h);

        // Utility structure
        let util: Utils = Utils::new();
        // Calculate colour if required here
        let cdlin = util.mon2lin(&self.basecolor);
        // Calculate lumincance approx
        let cdlum = 0.3_f64 * cdlin.x + 0.6_f64 * cdlin.y + 0.1_f64 * cdlin.z;

        let newvec = glm::DVec3::new(cdlin.x / cdlum, cdlin.y / cdlum, cdlin.z / cdlum);
        let ctint: glm::DVec3;
        ctint = if cdlum > 0.0_f64 {newvec} else {glm::DVec3::new(1.0, 1.0, 1.0)};
        
        // TODO Finish functions from here


        return glm::DVec3::new(0.0, 0.0, 0.0);
    }
}