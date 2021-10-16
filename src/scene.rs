use crate::object::{DrawError, Object, ObjectDrawData};
use crate::path_trace::intersectable::{IntersectInfo, Intersectable};
use crate::path_trace::ray::Ray;
use crate::rasterize::drawable::Drawable;

pub struct Scene<'a> {
    objects: Vec<Box<dyn Object<'a>>>,
}

impl Default for Scene<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Scene<'a> {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: Box<dyn Object<'a>>) {
        self.objects.push(object);
    }

    pub fn get_objects(&self) -> &Vec<Box<dyn Object<'a>>> {
        &self.objects
    }
}

impl Intersectable for Scene<'_> {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<IntersectInfo> {
        let hit_infos: Vec<_> = self
            .objects
            .iter()
            .map(|object| object.hit(ray, t_min, t_max))
            .filter(|object| object.is_some())
            .collect();

        if hit_infos.is_empty() {
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

        res
    }
}

impl<'a> Drawable<ObjectDrawData<'a>, DrawError> for Scene<'a> {
    fn draw(&self, extra_data: &mut ObjectDrawData<'a>) -> Result<(), DrawError> {
        self.objects
            .iter()
            .try_for_each(|object| object.draw(extra_data))
    }
}
