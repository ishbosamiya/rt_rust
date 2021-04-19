use crate::math::Vec3;
use crate::ray::Ray;

pub struct IntersectInfo {
    point: Vec3,
    normal: Vec3,
}

impl IntersectInfo {
    pub fn new(point: Vec3, normal: Vec3) -> Self {
        return Self { point, normal };
    }

    pub fn get_point(&self) -> &Vec3 {
        return &self.point;
    }

    pub fn get_normal(&self) -> &Vec3 {
        return &self.normal;
    }
}

pub trait Intersectable {
    fn hit(&self, ray: &Ray) -> Option<IntersectInfo>;
}
