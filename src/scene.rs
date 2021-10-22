use crate::bvh::BVHTree;
use crate::object::{DrawError, Object, ObjectDrawData};
use crate::path_trace::intersectable::{IntersectInfo, Intersectable};
use crate::path_trace::ray::Ray;
use crate::rasterize::drawable::Drawable;

pub struct Scene {
    objects: Vec<Box<dyn Object>>,

    /// BVH over all the objects in the scene. User must handle
    /// building/rebuilding the bvh before usage.
    bvh: Option<BVHTree<usize>>,

    /// true if model matrices are currently applied
    model_matrices_applied: bool,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            bvh: None,
            model_matrices_applied: false,
        }
    }

    pub fn add_object(&mut self, object: Box<dyn Object>) {
        self.objects.push(object);
    }

    pub fn get_objects(&self) -> &Vec<Box<dyn Object>> {
        &self.objects
    }

    pub fn get_objects_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.objects
    }

    pub fn apply_model_matrices(&mut self) {
        if self.model_matrices_applied {
            return;
        }
        self.objects.iter_mut().for_each(|object| {
            object.apply_model_matrix();
        });
        self.model_matrices_applied = true;
    }

    pub fn unapply_model_matrices(&mut self) {
        if !self.model_matrices_applied {
            return;
        }
        self.objects.iter_mut().for_each(|object| {
            object.unapply_model_matrix();
        });
        self.model_matrices_applied = false;
    }

    pub fn build_bvh(&mut self, epsilon: f64) {
        let mut bvh = BVHTree::new(self.objects.len(), epsilon, 4, 8);

        self.objects.iter().enumerate().for_each(|(index, object)| {
            let co = object.get_min_max_bounds();
            let co = [co.0, co.1];
            bvh.insert(index, &co);
        });

        bvh.balance();

        self.bvh = Some(bvh);
    }
}

impl Intersectable for Scene {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<IntersectInfo> {
        assert!(self.model_matrices_applied);
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

impl Drawable for Scene {
    type ExtraData = ObjectDrawData;
    type Error = DrawError;

    fn draw(&self, extra_data: &mut Self::ExtraData) -> Result<(), DrawError> {
        unsafe {
            extra_data.set_use_model_matrix(!self.model_matrices_applied);
        }
        self.get_objects()
            .iter()
            .try_for_each(|object| object.draw(extra_data))?;
        Ok(())
    }
}
