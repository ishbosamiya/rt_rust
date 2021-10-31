use std::cell::RefCell;
use std::rc::Rc;

use crate::path_trace::intersectable::{IntersectInfo, Intersectable};
use crate::path_trace::ray::Ray;
use crate::rasterize::gpu_utils::draw_smooth_sphere_at;
use crate::rasterize::{drawable::Drawable, gpu_immediate::GPUImmediate};
use crate::util::vec3_apply_model_matrix;
use crate::{glm, path_trace};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Sphere {
    center: glm::DVec3,
    radius: f64,
}

impl Sphere {
    pub fn new(center: glm::DVec3, radius: f64) -> Self {
        Self { center, radius }
    }

    pub fn get_center(&self) -> &glm::DVec3 {
        &self.center
    }

    pub fn get_radius(&self) -> f64 {
        self.radius
    }

    pub fn apply_model_matrix(&mut self, model: &glm::DMat4) {
        self.center = vec3_apply_model_matrix(self.get_center(), model);
    }
}

impl Intersectable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<IntersectInfo> {
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
        let mut info = IntersectInfo::new(
            t,
            intersect_point,
            path_trace::direction_to_equirectangular(&(intersect_point - self.get_center())),
        );
        info.set_normal(ray, &outward_normal);

        Some(info)
    }
}

pub struct SphereDrawData {
    imm: Rc<RefCell<GPUImmediate>>,

    model_matrix: glm::DMat4,
    outside_color: glm::Vec4,
    inside_color: glm::Vec4,
}

impl SphereDrawData {
    pub fn new(
        imm: Rc<RefCell<GPUImmediate>>,
        model_matrix: glm::DMat4,
        outside_color: glm::Vec4,
        inside_color: glm::Vec4,
    ) -> Self {
        Self {
            imm,
            model_matrix,
            outside_color,
            inside_color,
        }
    }
}

impl Drawable for Sphere {
    type ExtraData = SphereDrawData;
    type Error = ();

    fn draw(&self, extra_data: &mut SphereDrawData) -> Result<(), ()> {
        draw_smooth_sphere_at(
            vec3_apply_model_matrix(&self.center, &extra_data.model_matrix),
            self.radius,
            extra_data.outside_color,
            extra_data.inside_color,
            &mut extra_data.imm.borrow_mut(),
        );
        Ok(())
    }

    fn draw_wireframe(&self, _extra_data: &mut SphereDrawData) -> Result<(), ()> {
        unreachable!("No Wireframe drawing for Sphere");
    }
}
