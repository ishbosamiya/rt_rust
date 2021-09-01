use crate::math::{Scalar, Vec3};
use crate::ray::Ray;
use nalgebra_glm as glm;
use std::cmp;
use crate::intersectable::{IntersectInfo};

/// Structure for implementing scattering on surface
pub struct SurfaceScatterEvent {
    info : IntersectInfo,
    wi : Vec3,
    wo : Vec3,
    weight : Vec3
    pdf : f64,
    flipped : bool,
    /// TBD Implement these structures
    ///sampler : PathSampleGenerator 
    ///reqLobe : BsdfLobes,
    ///sampledLobe : BsdfLobes,
    ///tangentFrame : TangentFrame
}

impl SurfaceScatterEvent {
    pub fn makeForward() {
        self.wo = -1 * self.wi;
        ///self.reqLobe = BsdfLobes::ForwardLobe (From enum)
    }
}