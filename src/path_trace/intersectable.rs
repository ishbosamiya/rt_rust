use super::{ray::Ray, shader_list::ShaderID};
use crate::{glm, object::ObjectID};

#[derive(Debug, Copy, Clone)]
pub struct IntersectInfo {
    t: f64,
    point: glm::DVec3,
    object_id: Option<ObjectID>,
    shader_id: Option<ShaderID>,
    normal: Option<glm::DVec3>,
    front_face: bool,
}

impl IntersectInfo {
    pub fn new(t: f64, point: glm::DVec3) -> Self {
        Self {
            t,
            point,
            object_id: None,
            shader_id: None,
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

    pub fn set_object_id(&mut self, object_id: ObjectID) {
        self.object_id = Some(object_id);
    }

    pub fn get_object_id(&self) -> Option<ObjectID> {
        self.object_id
    }

    pub fn set_shader_id(&mut self, shader_id: Option<ShaderID>) {
        self.shader_id = shader_id;
    }

    pub fn get_shader_id(&self) -> Option<ShaderID> {
        self.shader_id
    }

    pub fn get_normal(&self) -> &Option<glm::DVec3> {
        &self.normal
    }

    pub fn get_front_face(&self) -> bool {
        self.front_face
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
