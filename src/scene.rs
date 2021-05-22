use crate::intersectable::{IntersectInfo, Intersectable};
use crate::math::Scalar;
use crate::ray::Ray;

type Object = Box<dyn Intersectable + Send + Sync>;

pub struct Scene {
    objects: Vec<Object>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }

    pub fn get_objects(&self) -> &Vec<Object> {
        return &self.objects;
    }
}

impl Intersectable for Scene {
    fn hit(&self, ray: &Ray, t_min: Scalar, t_max: Scalar) -> Option<IntersectInfo> {
        let hit_infos: Vec<_> = self
            .objects
            .iter()
            .map(|object| object.hit(ray, t_min, t_max))
            .filter(|object| object.is_some())
            .collect();

        if hit_infos.len() == 0 {
            return None;
        }

        let mut res = hit_infos[0];
        let mut min = t_max;
        for info in hit_infos {
            if info.unwrap().get_t() < min {
                min = info.unwrap().get_t();
                res = info;
            }
        }

        return res;
    }
}
