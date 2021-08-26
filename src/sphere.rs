use crate::intersectable::{IntersectInfo, Intersectable};
use crate::math::{Scalar, Vec3};
use crate::ray::Ray;

use nalgebra_glm as glm;

pub struct Sphere {
    center: Vec3,
    radius: Scalar,
}

impl Sphere {
    pub fn new(center: Vec3, radius: Scalar) -> Self {
        Self { center, radius }
    }

    pub fn get_center(&self) -> &Vec3 {
        &self.center
    }

    pub fn get_radius(&self) -> Scalar {
        self.radius
    }
}

impl Intersectable for Sphere {
    fn hit(&self, ray: &Ray, t_min: Scalar, t_max: Scalar) -> Option<IntersectInfo> {
        let oc = ray.get_origin() - self.get_center();
        let a = glm::length2(ray.get_direction());
        let half_b = oc.dot(ray.get_direction());
        let c = glm::length2(&oc) - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();
        let mut root = (-half_b - sqrt_d) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrt_d) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let t = root;
        let intersect_point = ray.at(t);
        let outward_normal = (intersect_point - self.get_center()) / self.get_radius();
        let mut info = IntersectInfo::new(t, intersect_point);
        info.set_normal(ray, &outward_normal);

        Some(info)
    }
}
