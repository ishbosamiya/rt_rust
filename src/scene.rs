// use std::marker::PhantomData;

use std::cell::RefCell;
use std::rc::Rc;

use crate::object::{DrawError, Object, ObjectDrawData};
use crate::path_trace::intersectable::{IntersectInfo, Intersectable};
use crate::path_trace::ray::Ray;
use crate::rasterize::drawable::Drawable;
use crate::rasterize::gpu_immediate::GPUImmediate;

pub struct Scene<'a> {
    objects: Vec<Box<dyn Object<'a, ExtraData = ObjectDrawData<'a>, Error = DrawError>>>,
    imm: Rc<RefCell<GPUImmediate>>,
    // self_reference: PhantomData<&'s Self>,
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
            imm: Rc::new(RefCell::new(GPUImmediate::new())),
            // self_reference: PhantomData::default(),
        }
    }

    pub fn add_object(
        &mut self,
        object: Box<dyn Object<'a, ExtraData = ObjectDrawData<'a>, Error = DrawError>>,
    ) {
        self.objects.push(object);
    }

    pub fn get_objects(
        &self,
    ) -> &Vec<Box<dyn Object<'a, ExtraData = ObjectDrawData<'a>, Error = DrawError>>> {
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

impl<'a> Drawable<'a> for Scene<'a> {
    type ExtraData = ();
    type Error = DrawError;

    fn draw(&'a self, _extra_data: &mut Self::ExtraData) -> Result<(), DrawError> {
        let mut imm = self.imm.borrow_mut();
        let mut draw_data = ObjectDrawData::new(&mut imm);
        self.get_objects()
            .iter()
            .try_for_each(|object| object.draw(&mut draw_data))?;
        Ok(())
    }
}
