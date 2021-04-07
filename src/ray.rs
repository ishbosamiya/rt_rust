use nalgebra_glm as glm;

pub struct Ray {
    origin: glm::DVec3,
    direction: glm::DVec3,
}

impl Ray {
    pub fn new(origin: glm::DVec3, direction: glm::DVec3) -> Self {
        return Self { origin, direction };
    }

    pub fn get_origin(&self) -> &glm::DVec3 {
        return &self.origin;
    }

    pub fn get_direction(&self) -> &glm::DVec3 {
        return &self.direction;
    }
}
