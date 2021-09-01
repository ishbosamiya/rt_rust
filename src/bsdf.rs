use crate::math::{Scalar,Vec3};
use crate::ray::Ray;
use crate::subsurfacescatter::{SubsurfaceScatterEvent};

use nalgebra_glm as glm;


/// Geometric Data for normals and dot products
/// Removed front and back facing as it is already present in intersect info
pub struct GeomData {
    F : Option<Vec3>,
    V : Vec3,
    N : Vec3,
    H : Vec3,
    L : Vec3,                   /// L = I or incident light direction
    NdotL : f64,
    NdotV : f64,
    LdotH : f64,
    NdotH : f64,
    VdotH : f64
}

impl GeomData {
    pub fun calc_dots(&mut self) {
        self.NdotL = N.dot(L);
        self.NdotV = N.dot(V);
        self.LdotH = L.dot(H);
        self.NdotH = N.dot(H);
        self.VdotH = V.dot(H);
    }
}

/// Structure for the material
pub struct Material {
    metallness : f64,
    baseColor : glm::vec3,
    emissive : Option<glm::vec3>,
    roughness : f64,
    opacity : f64
}


/// Main trait for implementing the BSDF
pub trait BSDFData {
    ///pub fn new(material : &Material, data : &GeomData, alpha : f64, alpha_squared : f64) -> Self;

    fn sample(event : &SubsurfaceScatterEvent) -> bool;

    ///pub fn invert(samplet : &PathSampleGenerator) -> bool;

    pub fn eval(event : &SubsurfaceScatterEvent) -> Vec3;

    pub fn pdf(event : &SubsurfaceScatterEvent) -> f64;
}