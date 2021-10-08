use crate::glm;
use lerp::Lerp;

pub struct Utils {}

// Utility Functions
impl Utils {
    pub fn new() -> Self {
        Utils {}
    }
    pub fn schlick_fresnel(&self, u : f64) -> f64 {
        let m = u.clamp(0.0,1.0);
        let m2 = m * m;
        return m2 * m2 * m;
    }
    pub fn gtr1(&self, ndot_h : f64, a : f64) -> f64 {
        let pi = 3.14159265358979323846;
        if a >= 1.0 {
            return 1.0_f64 / pi;
        }
        let a2 = a.powf(2.0);
        let t = 1.0_f64 + (a2 - 1.0_f64) * ndot_h * ndot_h;
        // Check again if it is log10 or log2
        return (a2 - 1.0_f64) / (pi * a2.log10() * t);
    }
    pub fn gtr2_aniso(&self, ndot_h : f64, hdot_x : f64, hdot_y : f64, ax : f64, ay : f64) -> f64 {
        let pi = 3.14159265358979323846;
        return 1.0_f64 / (pi * ax * ay * ((hdot_x / ax).powf(2.0) + (hdot_y / ay).powf(2.0) + ndot_h * ndot_h ).powf(2.0));
    }
    pub fn smithg_ggx(&self, ndot_v : f64, alphag : f64) -> f64 {
        let a = alphag.powf(2.0);
        let b = ndot_v.powf(2.0);
        return 1.0_f64 / (ndot_v + (a + b - a * b).sqrt());
    }
    pub fn smithg_ggx_aniso(&self, ndot_v : f64, vdot_x : f64, vdot_y : f64, ax : f64, ay : f64) -> f64 {
        return 1.0_f64 / (ndot_v + ((vdot_x * ax).powf(2.0) + (vdot_y * ay).powf(2.0)).sqrt());
    }
    pub fn mon2lin(&self, x : &glm::DVec3) -> glm::DVec3 {
        return glm::DVec3::new(x.x.powf(2.2), x.y.powf(2.2), x.z.powf(2.2));
    }
    pub fn mix(&self, x: &glm::DVec3, y: &glm::DVec3,z: f64) -> glm::DVec3 {
        return x.lerp(y,z);
    }

    pub fn mixnum(&self, x: f64,y: f64,z: f64) -> f64 {
        return x.lerp(y,z);
    }
}