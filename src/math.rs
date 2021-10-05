use rand::prelude::*;

use crate::glm;

pub fn random_in_unit_sphere() -> glm::DVec3 {
    loop {
        let p = glm::vec3(random(), random(), random()) * 2.0 - glm::vec3(1.0, 1.0, 1.0);
        if glm::length2(&p) >= 1.0 {
            continue;
        }
        return p;
    }
}
