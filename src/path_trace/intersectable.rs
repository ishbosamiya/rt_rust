use super::ray::Ray;
use crate::glm;

#[derive(Debug, Copy, Clone)]
pub struct IntersectInfo {
    t: f64,
    point: glm::DVec3,
    normal: Option<glm::DVec3>,
    front_face: bool,
}

impl IntersectInfo {
    pub fn new(t: f64, point: glm::DVec3) -> Self {
        Self {
            t,
            point,
            normal: None,
            front_face: false,
        }
    }

    pub fn get_t(&self) -> f64 {
        self.t
    }

    pub fn get_point(&self) -> &glm::DVec3 {
        &self.point
    }

    pub fn get_normal(&self) -> &Option<glm::DVec3> {
        &self.normal
    }

    /// Sets the normal and whether or not the hit was on the front
    /// face based on the true normal given and the ray's direction
    pub fn set_normal(&mut self, ray: &Ray, outward_normal: &glm::DVec3) {
        self.front_face = ray.get_direction().dot(outward_normal) < 0.0;
        if !self.front_face {
            self.normal = Some(-outward_normal);
        } else {
            self.normal = Some(*outward_normal);
        }
    }
}

pub trait Intersectable {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<IntersectInfo>;
}
