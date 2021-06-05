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

pub fn fresnel_calc(v : f64) {
    let u = 1.0 - v;
    let m = u.clamp(0.0, 1.0);
    let m2 = m * m;
    let mut f = m2 * m2 * m;

    return f;
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

pub struct Translucent {
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

pub struct Refraction {
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

<<<<<<< HEAD
=======

>>>>>>> 990055249be78b0ffd165786b4c39a6634e33ef3
    /// Function that evaluates the throughput for said shader
    fn eval(&mut self, backfacing : bool) -> Vec3 {
        let eta = 1e-5_f64;
        let weight = Vec3::new(1.0_f64, 1.0_f64, 1.0_f64);

        let mut ior = if backfacing {1.0_f64 / eta} else {eta};

        let mut cosno = self.N.dot(self.L);
        let fresnel = fresnel_dielectric(cosno, ior);

        let transmission = ((1 >> 8) & 0xFF) as f64;
        let mut diffuse_weight = (1.0_f64 - saturate(self.material.metallness)) * (1.0 - saturate(transmission));
        /// Specular weights not calculated
        
        /// Vector * Scalar * Vector (Check if it works?)
        let mut diff_weight = weight * diffuse_weight * self.material.baseColor;

        return diff_weight;
    }

    /// Evaluates eval and pdf for scatter ray
    fn eval_sample(&mut self, diffuse : &Vec3, pdf : &mut f64, eval : &mut Vec3, inward : &mut Ray) {
        let mut randu = 0.0_f64;
        let mut randv = 0.0_f64;
        const M_2PI_F = 6.2831853071795864_f64;
        const M_1_PI_F = 0.3183098861837067_f64;
        let N = self.data.N;    /// Normal vector

        /// TODO function for getting values of randu and randv(PMJ Sample)
        
        /// Sampling the hemishpere as is needed for disney diffusion
        let mut r = cmp::max(0.0_f64, 1.0_f64 - randu * randu);
        r.sqrt();
        let phi = M_2PI_F * randv;
        let x = r * phi.cos();
        let y = r * phi.sin();

        /// Making orthonormals
        let mut T : Vec3 = Vec3::new(0.0, 0.0, 0.0);
        if (N.x != N.y || N.x != N.z) {
            T = Vec3::new(N.z - N.y, N.x - N.z, N.y - N.x);
        }
        else {
            T = Vec3::new = Vec3::new(N.z - N.y, N.x + N.z, -N.y - N.x);
        }
        T = glm::normalize(T);

        let mut B = N.cross(T);
        

        inward = x * T + y * B + z * N;
        pdf = 0.5_f64 *  M_1_PI_F;

        /// Post Sampling hemisphere
        if (N.dot(inward) > 0) {
            let mut H = glm::normalize(self.data.L + inward);   /// L is incident light vector

            /// Calculate principled diffuse brdf part
            let ndotl = cmp::max(N.dot(inward), 0.0);
            let ndotv = cmp::max(N.dot(self.data.L), 0.0);

            if (ndotl < 0 || ndotv < 0) {
                pdf = 0.0;
                eval = Vec3::new(0.0, 0.0, 0.0);
            }
            else {
                let ldoth = inward.dot(H);
                let mut fl = fresnel_calc(ndotl);
                let mut fv = fresnel_calc(ndotv);

                let fd90 = 0.5 + 2.0 * ldoth * ldoth * self.material.roughness;
                let fd = (1.0 * (1.0 - fl) + fd90 * fl) * (1.0 * (1.0 - fv) + fd90 * fv);

                let value = M_1_PI_F * nodtl * fd;

                eval = Vec3::new(value, value, value);
            }
        }
        else {
            eval = Vec3::new(0.0, 0.0, 0.0);
        }
    }

    /// Returns the ray
    /// Function will also call the eval function and evaluate the throughput
    pub fn scatter_ray(&mut self, inward_ray : &Ray, outward_ray : &Ray, throughput : &Vec3, backfacing : bool) -> (Ray,Vec3) {
        /// Call the eval function
        let mut diffuse = eval(backfacing);
        let mut pdf = 0.0_f64;
        let mut eval = Vec3::new(0.0_f64, 0.0_f64, 0.0_f64);

        /// Samples the bsdf
        self.eval_sample(diffuse, &mut pdf, &mut eval, &mut inward_ray);

        if (pdf != glm::vec3(0.0_f64, 0.0_f64, 0.0_f64)) {
            /// Evaluating diffuse weight multiplied by the eval from above function
            eval = eval * diffuse;
        }

        /// Modify Throughput
        /// bsdf_bounce()

        return (outward_ray, throughput);
    }
}

/// TBD : FINISH THE FOLLOWING TRAITS
/// MINOR CHANGES REQUIRED FROM THE ABOVE
/// TEST DIFFUSE SHADER FOR ERRORS (WILL BE PRESENT)

impl BSDFData for Reflection {

}

impl BSDFData for Translucent {

}

impl BSDFData for Transparent {

}

impl BSDFData for Refraction {

}