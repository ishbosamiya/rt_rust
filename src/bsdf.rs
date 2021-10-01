use crate::math::{Vec3};
use nalgebra_glm as glm;


/// Geometric Data for normals and dot products
/// Removed front and back facing as it is already present in intersect info

/// Structure for the material
pub struct Material {
    metallness : f64,
    baseColor : Vec3,
    emissive : Option<Vec3>,
    roughness : f64,
    opacity : f64
}


/// Main trait for implementing the BSDF
pub trait BSDF {
    ///pub fn new(material : &Material, data : &GeomData, alpha : f64, alpha_squared : f64) -> Self;
    /// fn sample(event : &SubsurfaceScatterEvent) -> bool    
    fn eval(L : &Vec3, V : &Vec3, N : &Vec3, X : &Vec3, Y : &Vec3) -> Vec3;
}