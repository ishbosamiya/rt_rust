use nalgebra_glm as glm;
use rand::prelude::*;

pub type Vec3 = glm::DVec3;
pub type Scalar = f64;

pub fn random_in_unit_sphere() -> Vec3 {
    loop {
        let p = glm::vec3(random(), random(), random());
        if glm::length2(&p) >= 1.0 {
            continue;
        }
        return p;
    }
}
