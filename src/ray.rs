use crate::math::{Scalar, Vec3};

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        return Self { origin, direction };
    }

    pub fn get_origin(&self) -> &Vec3 {
        return &self.origin;
    }

    pub fn get_direction(&self) -> &Vec3 {
        return &self.direction;
    }

    pub fn at(&self, t: Scalar) -> Vec3 {
        return &self.origin + t * &self.direction;
    }
}
