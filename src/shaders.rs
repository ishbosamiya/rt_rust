use nalgebra_glm as glm
use crate::math::{Scalar,Vec3,saturate};
use crate::ray::Ray;
use crate::bsdf::{Material, BSDFData, GeomData};

/// Function for calculating the fresnel dielectric 
pub fun fresnel_dielectric(cosi : f64, eta : f64) -> f64 {
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


pub struct Diffuse {
    alpha : f64,
    alpha_squared : f64,
    material : Material,
    data : GeomData
}

/*
pub struct Reflection {
    alpha : f64,
    alpha_squared : f64,
    material : Material,
    data : &GeomData
}

pub struct Glossy {
    alpha : f64,
    alpha_squared : f64,
    material : Material,
    data : &GeomData
}

pub struct Transparent {
    alpha : f64,
    alpha_squared : f64,
    material : Material,
    data : &GeomData
}

*/

impl BSDFData for Diffuse {
    pub fn new(material : &Material, data : &GeomData, alpha : f64, alpha_squared : f64) -> Self {
        return Self {
            alpha,
            alpha_squared,
            material,
            data
        } 
    }

    /// Setting up all the dot products before hand
    pub fn init_data(&mut self) {
        self.data.calc_dots();
    }

    /// Function that evaluates the throughput for said shader
    fn eval(&mut self, backfacing : bool) -> Vec3 {
        let eta = 1e-5_f64;
        let weight = Vec3::new(1.0, 1.0, 1.0);

        let mut ior = if backfacing {1.0 / eta} else {eta};

        let mut cosno = self.N.dot(self.L);
        let fresnel = fresnel_dielectric(cosno, ior);

        let transmission = ((1 >> 8) & 0xFF);
        let mut diffuse_weight = (1.0 - saturate(self.material.metallness)) * (1.0 - saturate(transmission));
        /// Specular weights not calculated
        
        /// Vector * Scalar * Vector (Check if it works?)
        let mut diff_weight = weight * diffuse_weight * self.material.baseColor;

        return diff_weight;
    }

    /// Evaluates eval and pdf for scatter ray
    fn eval_sample(diffuse : &Vec3, pdf : &Vec3, eval : &Vec3, outward : &Ray) {

    }

    /// Returns the ray
    /// Function will also call the eval function and evaluate the throughput
    pub fn scatter_ray(&mut self, inward_ray : &Ray, outward_ray : &Ray, throughput : &Vec3, backfacing : bool) -> (Ray,Vec3) {
        /// Call the eval function
        let mut diffuse = eval(backfacing);
        let mut pdf = Vec3::new(0.0, 0.0, 0.0);
        let mut eval = Vec3::new(0.0, 0.0, 0.0);

        /// TODO COMPLETE THIS FUNCTION
        self.eval_sample(diffuse, pdf, eval, outward_ray);

        if (pdf != glm::vec3(0.0, 0.0, 0.0)) {
            /// Evaluating diffuse weight multiplied by the eval from above function
            eval = eval * diffuse;
        }

        /// Calculate the outward ray that is required
        throughput = throughput * eval;

        return (outward_ray, throughput);
    }
}

/// TBD : FINISH THE FOLLOWING TRAITS
/// MINOR CHANGES REQUIRED FROM THE ABOVE
/// TEST DIFFUSE SHADER FOR ERRORS (WILL BE PRESENT)

impl BSDFData for Reflection {

}

impl BSDFData for Glossy {

}

impl BSDFData for Transparent {

}