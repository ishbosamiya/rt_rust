use crate::math::{Vec3};
use nalgebra_glm as glm;
use crate::blinn::{Blinn};

// Removed front and back facing as it is already present in intersect info

// Structure for the Template
pub struct BSDFTemplate {
    baseColor : Vec3,
    roughness : f64,
    brightness : f64,
    opacity : f64
}

// Implementing the template with various functions
// Need to finish this function as provided in brdftemplatesphere.frag
// Need not put color as of now
impl BSDFTemplate {
    fn computeWithDirectionalLight(&self, surf : &Vec3, L : &Vec3, view : &Vec3, N : &Vec3, X : &Vec3, Y : &Vec3) -> Vec3 {
        let zerovec : Vec3 = Vec3::new(0.0_f64, 0.0_f64, 0.0_f64);
        let mut BlinnModel : Blinn = BSDF::new();
        let S = BlinnModel.eval(L, view, N, X, Y);
        let mut b = if S > zerovec {S} else {zerovec};

        b = b * N.dot(L);

        return b;
    }
    // Finish this function from main of above file
    fn setup(&self, ray : &Vec3, vertex : &Vec3) -> Vec3 {
        // Replace ray with vertex vector in world space
        // Ray then becomes incident light vector
        let normal = vertex.normalize();
        let tangent = (Vec3::new(0.0_f64, 1.0_f64, 0.0_f64).cross(&normal)).normalize();
        let bitangent = (normal.cross(&tangent)).normalize();

        let surfacepos : Vec3 = vertex.normalize();

        let viewvec = Vec3::new(0.0_f64, 0.0_f64, 1.0_f64);
        let mut b = self.computeWithDirectionalLight(&surfacepos, ray, &viewvec, &normal, &tangent, &bitangent);

        b = b * self.brightness;

        // Calculate exposure - TBD
        // b = b * self.opacity.powf(2.0)
        // Check if gamma is roughness or not

        let invgamma = 1.0_f64 / self.roughness;
        
        let mut new : Vec3;
        new = Vec3::new(b.x.powf(invgamma), b.y.powf(invgamma), b.z.powf(invgamma));

        return new;
    }
}


// Main trait for implementing the BSDF
pub trait BSDF {
    fn new() -> Self;
    // fn sample(event : &SubsurfaceScatterEvent) -> bool    
    fn eval(&self, L : &Vec3, V : &Vec3, N : &Vec3, X : &Vec3, Y : &Vec3) -> Vec3;
}