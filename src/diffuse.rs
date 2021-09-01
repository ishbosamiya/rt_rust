use nalgebra_glm as glm
use std::cmp
use crate::math::{Scalar,Vec3,saturate};
use crate::ray::Ray;
use crate::bsdf::{Material, BSDFData, GeomData};

/// Function for calculating the fresnel dielectric 
pub fn fresnel_dielectric(cosi : f64, eta : f64) -> f64 {
    let c = cosi.abs();
    let mut g = eta * eta - 1 + c * c;
    if (g > 0) {
        g = g.sqrt();
        let a = (g - c) / (g + c);
        let  b = (c * (g + c) - 1) / (c * (g - c) + 1);

        return 0.5 * a * a * (1 + b * b);
    } 

    return 1.0;
}