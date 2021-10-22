use crate::bvh::{BVHTree, RayHitData, RayHitOptionalData};
use crate::glm;
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
        assert!(self.bvh.is_some());

        let scene_ray_cast_callback = |(co, dir): (&glm::DVec3, &glm::DVec3),
                                       object_index: usize| {
            debug_assert_eq!(ray.get_origin(), co);
            debug_assert_eq!(ray.get_direction(), dir);

            let object = &self.objects[object_index];

            object.hit(ray, t_min, t_max).and_then(
                |info| -> Option<RayHitData<usize, IntersectInfo>> {
                    if info.get_t() > t_min && info.get_t() < t_max {
                        let mut hit_data = RayHitData::new(info.get_t());
                        hit_data.normal = *info.get_normal();
                        hit_data
                            .set_data(RayHitOptionalData::new(object_index, ray.at(info.get_t())));
                        hit_data.set_extra_data(info);
                        Some(hit_data)
                    } else {
                        None
                    }
                },
            )
        };

        self.bvh
            .as_ref()
            .unwrap()
            .ray_cast(
                *ray.get_origin(),
                *ray.get_direction(),
                Some(&scene_ray_cast_callback),
            )
            .map(|hit_data| hit_data.extra_data.unwrap())
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
