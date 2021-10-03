use crate::math::{Vec3};
use nalgebra_glm as glm;

/// Removed front and back facing as it is already present in intersect info

/// Structure for the Template
pub struct BSDFTemplate {
    metallness : f64,
    baseColor : Vec3,
    emissive : Option<Vec3>,
    roughness : f64,
    brightness : f64,
    opacity : f64
}

// Implementing the template with various functions
// Need to finish this function as provided in brdftemplatesphere.frag
// Need not put color as of now
impl BSDFTemplate {
    fn computeWithDirectionalLight(&self, surf : &Vec3, L : &Vec3, view : &Vec3, N : &Vec3, X : &Vec3, Y : &Vec3) {

    }
    fn setup(&self) {

    }
}


// Main trait for implementing the BSDF
pub trait BSDF {
    //pub fn new(material : &Material, data : &GeomData, alpha : f64, alpha_squared : f64) -> Self;
    // fn sample(event : &SubsurfaceScatterEvent) -> bool    
    fn eval(&self, L : &Vec3, V : &Vec3, N : &Vec3, X : &Vec3, Y : &Vec3) -> Vec3;
}