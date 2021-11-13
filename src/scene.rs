use std::cell::RefCell;
use std::collections::{hash_map, HashMap};
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
use crate::ui::DrawUI;
use crate::UiData;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(from = "SceneShadow")]
pub struct Scene {
    /// list of all objects indexed by their ObjectID
    objects: HashMap<ObjectID, Box<dyn Object>>,
    /// list of all object ids in the order of addition of objects
    object_ids: Vec<ObjectID>,

    /// BVH over all the objects in the scene. User must handle
    /// building/rebuilding the bvh before usage.
    bvh: Option<BVHTree<ObjectID>>,

    /// true if model matrices are currently applied
    model_matrices_applied: bool,

    /// selected object
    selected_object: Option<ObjectID>,
}

/// A shadow structure that is used to deserialize [`Scene`] and make
/// additional changes immediately after deserialization. Do not use
/// for anything other than deserialization.
///
/// This would no longer be required once something like a `finalizer`
/// attribute in `serde` is implemented. See
/// https://github.com/serde-rs/serde/issues/642 and similar for more
/// details. This workaround is based on the discussion on that issue.
#[derive(Debug, Serialize, Deserialize)]
struct SceneShadow {
    objects: HashMap<ObjectID, Box<dyn Object>>,
    #[serde(default)]
    object_ids: Vec<ObjectID>,
    bvh: Option<BVHTree<ObjectID>>,
    model_matrices_applied: bool,
    selected_object: Option<ObjectID>,
}

impl From<SceneShadow> for Scene {
    fn from(scene_shadow: SceneShadow) -> Self {
        let object_ids = if scene_shadow.object_ids.is_empty() {
            scene_shadow
                .objects
                .iter()
                .sorted_by(|object1, object2| {
                    object1.1.get_object_name().cmp(object2.1.get_object_name())
                })
                .map(|(&object_id, _)| object_id)
                .collect()
        } else {
            scene_shadow.object_ids
        };
        Self {
            objects: scene_shadow.objects,
            object_ids,
            bvh: scene_shadow.bvh,
            model_matrices_applied: scene_shadow.model_matrices_applied,
            selected_object: scene_shadow.selected_object,
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            object_ids: Vec::new(),
            bvh: None,
            model_matrices_applied: false,
            selected_object: None,
        }
    }

    pub fn add_object(&mut self, mut object: Box<dyn Object>) {
        let object_id = unsafe { ObjectID::from_raw(rand::random()) };
        object.set_object_id(object_id);
        self.objects.insert(object_id, object);
        self.object_ids.push(object_id);
        self.bvh = None;
    }

    pub fn delete_object(&mut self, object_id: ObjectID) -> Option<Box<dyn Object>> {
        self.object_ids.remove(
            self.object_ids
                .iter()
                .enumerate()
                .find(|(_, id)| object_id == **id)
                .unwrap()
                .0,
        );
        let object = self.objects.remove(&object_id);
        if object.is_some() {
            self.bvh = None;
        }
        object
    }

    pub fn get_objects(&self) -> hash_map::Values<'_, ObjectID, Box<dyn Object>> {
        self.objects.values()
    }

    pub fn get_object(&self, object_id: ObjectID) -> Option<&dyn Object> {
        self.objects.get(&object_id).map(|object| object.as_ref())
    }

    /// Get mutable access to all objects of the scene as an
    /// iterator. Caller must ensure that BVH is rebuilt if necessary.
    pub fn get_objects_mut(&mut self) -> hash_map::ValuesMut<'_, ObjectID, Box<dyn Object>> {
        self.objects.values_mut()
    }

    pub fn get_object_mut(&mut self, object_id: ObjectID) -> Option<&mut Box<dyn Object>> {
        self.objects.get_mut(&object_id)
    }

    pub fn apply_model_matrices(&mut self) {
        if self.model_matrices_applied {
            return;
        }
        self.get_objects_mut().for_each(|object| {
            object.apply_model_matrix();
        });
        self.model_matrices_applied = true;
    }

    pub fn unapply_model_matrices(&mut self) {
        if !self.model_matrices_applied {
            return;
        }
        self.get_objects_mut().for_each(|object| {
            object.unapply_model_matrix();
        });
        self.model_matrices_applied = false;
    }

    pub fn build_bvh(&mut self, epsilon: f64) {
        let mut bvh = BVHTree::new(self.objects.len(), epsilon, 4, 8);

        self.get_objects().for_each(|object| {
            let co = object.get_min_max_bounds();
            let co = [co.0, co.1];
            bvh.insert(object.get_object_id(), &co);
        });

        bvh.balance();

        self.bvh = Some(bvh);
    }

    pub fn rebuild_bvh_if_needed(&mut self, epsilon: f64) {
        if self.bvh.is_none() {
            self.build_bvh(epsilon);
        }
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

    pub fn try_select_object(&mut self, ray: &Ray) {
        if let Some(info) = self.hit(ray, 0.01, 1000.0) {
            let object_id = info.get_object_id().unwrap();

            self.selected_object = Some(object_id);
        }
    }

    /// Get scene's selected object.
    pub fn get_selected_object(&self) -> Option<ObjectID> {
        self.selected_object
    }

    /// Get a reference to the scene's object ids.
    pub fn get_object_ids(&self) -> &[ObjectID] {
        self.object_ids.as_slice()
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
                |(co, dir): (&glm::DVec3, &glm::DVec3), object_id: ObjectID| {
                    debug_assert_eq!(ray.get_origin(), co);
                    debug_assert_eq!(ray.get_direction(), dir);

                    let object = &self.objects.get(&object_id).unwrap();

                    object.hit(ray, t_min, t_max).and_then(
                        |info| -> Option<RayHitData<ObjectID, IntersectInfo>> {
                            if info.get_t() > t_min && info.get_t() < t_max {
                                let mut hit_data = RayHitData::new(info.get_t());
                                hit_data.normal = *info.get_normal();
                                hit_data.set_data(RayHitOptionalData::new(
                                    object_id,
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
        self.get_objects().try_for_each(|object| {
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

impl DrawUI for Scene {
    type ExtraData = UiData;

    fn draw_ui(&self, _ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {}

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {
        let mut selected_object = self.get_selected_object();
        self.get_object_ids().iter().for_each(|&object_id| {
            let object = self.get_object(object_id).unwrap();
            let selected = match selected_object {
                Some(object_id) => object_id == object.get_object_id(),
                None => false,
            };
            let response = ui.selectable_label(selected, object.get_object_name());

            if response.clicked() {
                selected_object = Some(object.get_object_id());
            }
        });
        self.selected_object = selected_object;

        if let Some(object_id) = self.get_selected_object() {
            if ui.button("Delete selected object").clicked() {
                self.selected_object = None;
                self.delete_object(object_id);
            }
            if ui.button("Deselect object").clicked() {
                self.selected_object = None;
            }
        } else {
            ui.label("No object currently selected");
        }
    }
}
