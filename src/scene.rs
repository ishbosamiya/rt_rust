use crossbeam::thread;

use crate::intersectable::{IntersectInfo, Intersectable};
use crate::math::Scalar;
use crate::ray::Ray;

type Object = Box<dyn Intersectable + Send + Sync>;

pub struct Scene {
    objects: Vec<Object>,
    num_threads: usize,
}

impl Scene {
    pub fn new(num_threads: usize) -> Self {
        Self {
            objects: Vec::new(),
            num_threads,
        }
    }

    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }

    pub fn hit(&'static self, ray: &Ray, t_min: Scalar, t_max: Scalar) -> Option<IntersectInfo> {
        let mut chunk_size = self.objects.len() / self.num_threads;
        if chunk_size == 0 {
            chunk_size = 1;
        }
        let ray = *ray;

        let mut hit_infos = Vec::new();
        thread::scope(|scope| {
            let mut handles = Vec::new();

            for objects in self.objects.chunks(chunk_size) {
                let handle = scope.spawn(move |_| {
                    let mut hit_infos = Vec::new();

                    for object in objects {
                        hit_infos.push(object.hit(&ray, t_min, t_max));
                    }
                    return hit_infos;
                });
                handles.push(handle);
            }

            for handle in handles {
                let infos = handle.join().unwrap();
                hit_infos.extend_from_slice(&infos);
            }
        })
        .unwrap();

        let hit_infos: Vec<_> = hit_infos.iter().filter(|info| info.is_some()).collect();

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

        return *res;
    }
}
