use crate::math::{Vec3};
use crate::bsdf::{BSDF};


pub struct Blinn {}

impl BSDF for Blinn {
    fn new() -> Self {
        Blinn {}
    }

    fn eval(&self, L : &Vec3, V : &Vec3, N : &Vec3, X : &Vec3, Y : &Vec3) -> Vec3 {
        let include_fresnel : bool = true;
        let divide_by_NdotL : bool = true;
        let S = L + V;
        let H = S.normalize();

        let NdotH = N.dot(&H);
        let VdotH = V.dot(&H);
        let NdotL = N.dot(L);
        let NdotV = N.dot(V);

        let x = NdotH.acos() * 100.0_f64;
        let D = ( -x * x).exp();
        let G : f64;
        if NdotV < NdotL {
            G = if ((2.0_f64 * NdotV * NdotH < VdotH)) {2.0_f64 * NdotH / VdotH} else {1.0_f64 / NdotV};
        }
        else {
            G = if ((2.0_f64 * NdotL * NdotH < VdotH)) {2.0_f64 * NdotH * NdotL / (VdotH * NdotV)} else {1.0_f64 / NdotV};
        }
        let c = VdotH;
        let g = (2.5_f64 * 2.5_f64 + c * c - 1.0_f64).sqrt();
        let mut F : f64 = 0.0_f64;
        // Double Check this value
        F = 0.5_f64 * ((g - c) * (g - c)) / ((g + c) * (g + c)) * (1.0_f64 + (c * (g + c) - 1.0_f64).powf(2.0)) / (c * (g - c) + 1.0_f64).powf(2.0);

        let mut val : f64;
        if NdotH < 0.0_f64 {
            val = 0.0_f64;
        }
        else {
            let fresnel = if include_fresnel {F} else {1.0_f64};
            val = D * G * fresnel;
        }

        if divide_by_NdotL {
            val = val / N.dot(L);
        }
        return Vec3::new(val, val, val);

    }
}