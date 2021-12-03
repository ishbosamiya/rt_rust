use std::{collections::HashMap, convert::TryInto};

use embree_rust::{GeometryID, SceneID};
use itertools::Itertools;

use crate::{
    glm,
    mesh::Mesh,
    object::{Object, ObjectID},
    path_trace::{
        intersectable::{IntersectInfo, Intersectable},
        ray::Ray,
    },
    sphere::Sphere,
};

#[derive(Debug)]
pub struct Embree {
    embree: embree_rust::Embree,
    /// ID of the scene created by Embree, currently supports only one
    /// scene
    scene_id: Option<SceneID>,
    /// Mapping from embree's [`GeometryID`] to [`ObjectID`]
    geometry_ids: HashMap<GeometryID, ObjectID>,
}

impl Embree {
    pub fn new() -> Self {
        Self {
            embree: embree_rust::Embree::new(),
            scene_id: None,
            geometry_ids: HashMap::new(),
        }
    }

    pub fn commit_scene(&mut self) {
        let scene_id = self.get_scene_id();
        self.scene_id = Some(self.embree.commit_scene(scene_id));
    }

    pub fn add_object(&mut self, object: &dyn Object) {
        object.add_object_to_embree(self);
    }

    pub fn add_mesh(&mut self, mesh: &Mesh, object_id: ObjectID) {
        let scene_id = self.get_scene_id();

        let verts = mesh
            .get_vertices()
            .iter()
            .map(|vert| {
                let pos = vert.get_pos();
                embree_rust::Vert::new(embree_rust::Vec3::new(
                    pos[0] as f32,
                    pos[1] as f32,
                    pos[2] as f32,
                ))
            })
            .collect_vec();
        let triangles = mesh
            .get_faces()
            .iter()
            .flat_map(|face| {
                // TODO: need to find a better way to triangulate the
                // face

                // It doesn't make sense for a face to have only 2 verts
                assert!(face.len() > 2);

                let v1_index = face[0];
                face.iter()
                    .skip(1)
                    .tuple_windows()
                    .map(move |(&v2_index, &v3_index)| {
                        embree_rust::Triangle::new(
                            v1_index.try_into().unwrap(),
                            v2_index.try_into().unwrap(),
                            v3_index.try_into().unwrap(),
                        )
                    })
            })
            .collect_vec();

        let geometry_id = self.embree.add_geometry_triangle(&verts, &triangles);

        self.embree.attach_geometry_to_scene(geometry_id, scene_id);
        self.geometry_ids.insert(geometry_id, object_id);
    }

    pub fn add_sphere(&mut self, sphere: &Sphere, object_id: ObjectID) {
        let scene_id = self.get_scene_id();
        let geometry_id = self.embree.add_geometry_sphere(&[embree_rust::Sphere::new(
            embree_rust::Vec3::new(
                sphere.get_center()[0] as f32,
                sphere.get_center()[1] as f32,
                sphere.get_center()[2] as f32,
            ),
            sphere.get_radius() as f32,
        )]);
        self.embree.attach_geometry_to_scene(geometry_id, scene_id);
        self.geometry_ids.insert(geometry_id, object_id);
    }

    /// Get scene id, create new scene is necessary
    ///
    /// TODO: need a better way to handle this, the user must always
    /// have the choice of when to create a scene
    #[inline]
    fn get_scene_id(&mut self) -> SceneID {
        if self.scene_id.is_none() {
            self.scene_id = Some(self.embree.add_scene());
        }
        self.scene_id.unwrap()
    }
}

impl Intersectable for Embree {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<IntersectInfo> {
        let ray_hit = self.embree.intersect_scene(
            self.scene_id.expect("scene id must be available by now"),
            embree_rust::Ray::new(
                embree_rust::Vec3::new(
                    ray.get_origin()[0] as f32,
                    ray.get_origin()[1] as f32,
                    ray.get_origin()[2] as f32,
                ),
                t_min as f32,
                t_max as f32,
                embree_rust::Vec3::new(
                    ray.get_direction()[0] as f32,
                    ray.get_direction()[1] as f32,
                    ray.get_direction()[2] as f32,
                ),
                0.0,
            ),
        );

        if ray_hit.hit.geomID == embree_rust::INVALID_GEOMETRY_ID {
            None
        } else {
            let t = ray_hit.ray.tfar as f64;
            let mut info = IntersectInfo::new(t, ray.at(t), glm::zero());
            info.set_normal(
                ray,
                &glm::vec3(
                    ray_hit.hit.Ng_x as _,
                    ray_hit.hit.Ng_y as _,
                    ray_hit.hit.Ng_z as _,
                ),
            );
            Some(info)
        }
    }
}

impl Default for Embree {
    fn default() -> Self {
        Self::new()
    }
}
