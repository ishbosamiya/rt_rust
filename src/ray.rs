use crate::math::{Scalar, Vec3};

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    pub fn get_origin(&self) -> &Vec3 {
        &self.origin
    }

    pub fn get_direction(&self) -> &Vec3 {
        &self.direction
    }

    pub fn at(&self, t: Scalar) -> Vec3 {
        self.origin + t * self.direction
    }
}
