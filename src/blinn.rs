use crate::bsdf::BSDF;
use crate::glm;

pub struct Blinn {}

impl BSDF for Blinn {
    fn new() -> Self {
        Blinn {}
    }
    fn sample(&self, out: &glm::DVec3, vertex: &glm::DVec3) -> glm::DVec3 {
        return glm::zero();
    }

    fn eval(
        &self,
        l: &glm::DVec3,
        v: &glm::DVec3,
        n: &glm::DVec3,
        x: &glm::DVec3,
        y: &glm::DVec3,
    ) -> glm::DVec3 {
        let include_fresnel: bool = true;
        let divide_by_ndot_l: bool = true;
        let s = l + v;
        let h = s.normalize();

        let ndot_h = n.dot(&h);
        let vdot_h = v.dot(&h);
        let ndot_l = n.dot(l);
        let ndot_v = n.dot(v);

        let x_val = ndot_h.acos() * 100.0_f64;
        let d = (-x_val * x_val).exp();
        let g_val: f64;
        if ndot_v < ndot_l {
            g_val = if 2.0_f64 * ndot_v * ndot_h < vdot_h {
                2.0_f64 * ndot_h / vdot_h
            } else {
                1.0_f64 / ndot_v
            };
        } else {
            g_val = if 2.0_f64 * ndot_l * ndot_h < vdot_h {
                2.0_f64 * ndot_h * ndot_l / (vdot_h * ndot_v)
            } else {
                1.0_f64 / ndot_v
            };
        }
        let c = vdot_h;
        let g = (2.5_f64 * 2.5_f64 + c * c - 1.0_f64).sqrt();
        let f: f64;
        // Double Check this value
        f = 0.5_f64 * ((g - c) * (g - c)) / ((g + c) * (g + c))
            * (1.0_f64 + (c * (g + c) - 1.0_f64).powf(2.0))
            / (c * (g - c) + 1.0_f64).powf(2.0);

        let mut val: f64;
        if ndot_h < 0.0_f64 {
            val = 0.0_f64;
        } else {
            let fresnel = if include_fresnel { f } else { 1.0_f64 };
            val = d * g_val * fresnel;
        }

        if divide_by_ndot_l {
            val = val / n.dot(l);
        }
        return glm::vec3(val, val, val);
    }
}
