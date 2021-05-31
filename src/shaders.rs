use nalgebra_glm as glm
use crate::math::{Scalar,Vec3};
use crate::ray::Ray;
use crate::bsdf::{Material, BSDFData, GeomData};

pub struct Diffuse {
    alpha : f64,
    alpha_squared : f64,
    diffuse_reflectance : glm::vec3,
    data : &GeomData
}

pub struct GlossyReflection {
    data : &GeomData
}

pub struct MirrorReflection {
    alpha : f64,
    data : &GeomData
}

pub struct Refraction {
    data : &GeomData
}

impl BSDFData for Diffuse {
    pub fn init_shaders(&self) {
        /// Setting up all the dot products before hand
        self.data.calc_dots();
    }

    pub fn scatter_ray(inward_ray : &Vec3, outward_ray : &Vec3, throughput : &Vec3, material : &Material) -> Vec3 {

    }

    pub fn eval(N : &Vec3, L : &Vec3, V : &Vec3, H : &Vec3) -> Vec3 {

    }
}