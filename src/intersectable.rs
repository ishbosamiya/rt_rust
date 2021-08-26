use crate::math::{Scalar, Vec3};
use crate::ray::Ray;

#[derive(Debug, Copy, Clone)]
pub struct IntersectInfo {
    t: Scalar,
    point: Vec3,
    normal: Option<Vec3>,
    front_face: bool,
}

impl IntersectInfo {
    pub fn new(t: Scalar, point: Vec3) -> Self {
        Self {
            t,
            point,
            normal: None,
            front_face: false,
        }
    }

    pub fn get_t(&self) -> Scalar {
        self.t
    }

    pub fn get_point(&self) -> &Vec3 {
        &self.point
    }

    pub fn get_normal(&self) -> &Option<Vec3> {
        &self.normal
    }

    /// Sets the normal and whether or not the hit was on the front
    /// face based on the true normal given and the ray's direction
    pub fn set_normal(&mut self, ray: &Ray, outward_normal: &Vec3) {
        self.front_face = ray.get_direction().dot(outward_normal) < 0.0;
        if !self.front_face {
            self.normal = Some(-outward_normal);
        } else {
            self.normal = Some(*outward_normal);
        }
    }
}

pub trait Intersectable {
    fn hit(&self, ray: &Ray, t_min: Scalar, t_max: Scalar) -> Option<IntersectInfo>;
}
