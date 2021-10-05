use crate::glm;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    origin: glm::DVec3,
    direction: glm::DVec3,
}

impl Ray {
    pub fn new(origin: glm::DVec3, direction: glm::DVec3) -> Self {
        Self { origin, direction }
    }

    pub fn get_origin(&self) -> &glm::DVec3 {
        &self.origin
    }

    pub fn get_direction(&self) -> &glm::DVec3 {
        &self.direction
    }

    pub fn at(&self, t: f64) -> glm::DVec3 {
        self.origin + t * self.direction
    }
}
