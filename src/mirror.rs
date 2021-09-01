use nalgebra_glm as glm
use std::cmp
use crate::math::{Scalar,Vec3,saturate};
use crate::ray::Ray;
use crate::bsdf::{Material, BSDFData, GeomData};

let DiracAcceptanceThreshold = 1e-3_f64;

/// Helper Function to check for reflection constraint
pub fn checkRefractionConstraint(wi : &Vec3, wo : &Vec3, eta : f64, cosThetaT : f64) {
    let dotP : f64 = -wi.x * wo.x * eta - wi.y * wo.y * eta - (cosThetaT * wi.z) * wo.z;
    return abs(dotP - 1.0_f64) < DiracAcceptanceThreshold;
}


impl BSDFData for Mirror {
    fn sample(event : &SubsurfaceScatterEvent) -> bool {
        if (!event.requestedLobe.test(BsdfLobes::SpecularReflectionLobe))
            return False;
        event.wo = Vec3::new(-event.wi.x, -event.wi.y, event.wi.z);
        event.pdf = 1.0_f64;
        ///event.sampledLobe = BsdfLobes::SpecularReflectionLobe;
        /// Finish albedo function
        event.weight = albedo(event.info);
        return True;
    }

    ///pub fn invert(samplet : &PathSampleGenerator) -> bool;

    pub fn eval(event : &SubsurfaceScatterEvent) -> Vec3 {
        let evalR = event.requestedLobe.test(BsdfLobes::SpecularReflectionLobe);
        if (evalR && checkReflectionConstraint(event.wi, event.wo))
            return albedo(event.info);
        else
            return Vec3::new(0.0_f64, 0.0_f64, 0.0_f64);
    }

    pub fn pdf(event : &SubsurfaceScatterEvent) -> f64 {
        let mut sampleR = event.requestedLobe.test(BsdfLobes::SpecularReflectionLobe);
        if (sampleR && checkReflectionConstraint(event.wi, event.wo))
            return 1.0_f64;
        else
            return 0.0_f64;
    }
}