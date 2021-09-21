use nalgebra_glm as glm
use std::cmp
use crate::math::{Scalar,Vec3,saturate};
use crate::ray::Ray;
use crate::bsdf::{Material, BSDFData, GeomData};
use crate::subsurfacescatter::{SubsurfaceScatterEvent};

/// File for the Sampler
/// Contains Uniform path sampler and sample warp

pub fn invertPhi(w : &Vec3, mu : &f64) -> f64 {
    let INV_TWO_PI = 0.5_f64 * (1.0_f64 / 3.1415926536_f64)
    let result = if (w.x == 0.0 && w.y == 0.0) {mu*INV_TWO_PI} else {atan2(w.y, w.x) * INV_TWO_PI};
    if (result < Vec3::new(0.0, 0.0, 0.0))
        result += Vec3::new(1.0, 1.0, 1.0);
    return result;
}

pub fn uniformHemisphere(xi : &Vec2) -> Vec3 {
    let mut phi  = (2.0_f64 * 3.1415926536_f64) * xi.x;
    let mut r = sqrt(max(1.0 - xi.y * xi.y, 0.0));
    return Vec3::new(cos(phi)*r, sin(phi)*r, xi.y);
}

static inline float uniformHemispherePdf(p : &Vec3) {
    let INV_TWO_PI = 0.5_f64 * (1.0_f64 / 3.1415926536_f64);
    return INV_TWO_PI;
}


pub struct PathSampleGenerator {
    state : u64,
    sequence : u64
}

impl PathSampleGenerator {
    /// TBD Need an outstream module
    fn saveState(out : &outstream) {
        /// Write to file
    }
    /// PCG Random number generator
    fn next() -> u64 {
        let oldState = self.state;
        self.state = oldState * 6364136223846793005 + (self.sequence | 1);
        let mut xorshifted32 : u32 = ((oldState >> 18) ^ oldState) >> 27;
        let mut rot : u32 = oldState >> 59;
        return (xorshifted32 >> rot) | (xorshifted32 << (rot & 31))
    }
}