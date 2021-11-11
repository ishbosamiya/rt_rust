use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crate::bvh::BVHTree;
#[cfg(not(feature = "scene_no_bvh"))]
use crate::bvh::{RayHitData, RayHitOptionalData};
#[cfg(not(feature = "scene_no_bvh"))]
use crate::glm;
use crate::object::{DrawError, Object, ObjectDrawData, ObjectID};
use crate::path_trace::intersectable::{IntersectInfo, Intersectable};
use crate::path_trace::ray::Ray;
use crate::path_trace::shader_list::ShaderList;
use crate::rasterize::drawable::Drawable;
use crate::rasterize::gpu_immediate::GPUImmediate;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

// TODO: store Scene::objects in a HashMap instead of Vec for speed
// and object id stuff

#[derive(Debug, Serialize, Deserialize)]
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

    pub fn add_object(&mut self, mut object: Box<dyn Object>) {
        let object_id = unsafe { ObjectID::from_raw(rand::random()) };
        object.set_object_id(object_id);
        self.objects.push(object);
        self.bvh = None;
    }

    pub fn delete_object(&mut self, object_id: ObjectID) -> Option<Box<dyn Object>> {
        if let Some((index, _)) = self
            .objects
            .iter()
            .find_position(|object| object.get_object_id() == object_id)
        {
            self.bvh = None;
            Some(self.objects.remove(index))
        } else {
            None
        }
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

    pub fn get_min_max_bounds(&self) -> (glm::DVec3, glm::DVec3) {
        let bvh = self.bvh.as_ref().unwrap();
        bvh.get_min_max_bounds()

        // TODO: need to use the model matrices
        // self.objects.iter().fold(
        //     (
        //         glm::vec3(f64::MAX, f64::MAX, f64::MAX),
        //         glm::vec3(f64::MIN, f64::MIN, f64::MIN),
        //     ),
        //     |acc, object| {
        //         let (min, max) = object.get_min_max_bounds();
        //         (
        //             glm::vec3(
        //                 min[0].min(acc.0[0]),
        //                 min[1].min(acc.0[1]),
        //                 min[2].min(acc.0[2]),
        //             ),
        //             glm::vec3(
        //                 max[0].max(acc.1[0]),
        //                 max[1].max(acc.1[1]),
        //                 max[2].max(acc.1[2]),
        //             ),
        //         )
        //     },
        // )
    }
}

impl Intersectable for Scene {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<IntersectInfo> {
        assert!(self.model_matrices_applied);

        #[cfg(feature = "scene_no_bvh")]
        {
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

        #[cfg(not(feature = "scene_no_bvh"))]
        {
            assert!(self.bvh.is_some());

            let scene_ray_cast_callback =
                |(co, dir): (&glm::DVec3, &glm::DVec3), object_index: usize| {
                    debug_assert_eq!(ray.get_origin(), co);
                    debug_assert_eq!(ray.get_direction(), dir);

                    let object = &self.objects[object_index];

                    object.hit(ray, t_min, t_max).and_then(
                        |info| -> Option<RayHitData<usize, IntersectInfo>> {
                            if info.get_t() > t_min && info.get_t() < t_max {
                                let mut hit_data = RayHitData::new(info.get_t());
                                hit_data.normal = *info.get_normal();
                                hit_data.set_data(RayHitOptionalData::new(
                                    object_index,
                                    ray.at(info.get_t()),
                                ));
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
}

#[derive(Debug, Clone)]
pub struct SceneDrawData {
    imm: Rc<RefCell<GPUImmediate>>,
    shader_list: Arc<RwLock<ShaderList>>,
}

impl SceneDrawData {
    pub fn new(imm: Rc<RefCell<GPUImmediate>>, shader_list: Arc<RwLock<ShaderList>>) -> Self {
        Self { imm, shader_list }
    }
}

impl Drawable for Scene {
    type ExtraData = SceneDrawData;
    type Error = DrawError;

    fn draw(&self, extra_data: &mut Self::ExtraData) -> Result<(), DrawError> {
        let shader_list = extra_data.shader_list.read().unwrap();
        let mut object_draw_data = ObjectDrawData::new(extra_data.imm.clone(), glm::zero());
        unsafe {
            object_draw_data.set_use_model_matrix(!self.model_matrices_applied);
        }
        self.get_objects().iter().try_for_each(|object| {
            let viewport_color = object
                .get_path_trace_shader_id()
                .and_then(|shader_id| shader_list.get_shader(shader_id))
                .map_or_else(glm::zero, |shader| *shader.get_viewport_color());
            object_draw_data.set_viewport_color(glm::vec4(
                viewport_color[0],
                viewport_color[1],
                viewport_color[2],
                1.0,
            ));
            object.draw(&mut object_draw_data)
        })?;
        Ok(())
    }
}
