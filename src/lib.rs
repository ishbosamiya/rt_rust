pub mod camera;
pub mod image;
pub mod intersectable;
pub mod math;
pub mod ray;
pub mod scene;
pub mod sphere;
pub mod threadpool;

#[cfg(test)]
mod tests {
    #[test]
    fn single_ray_vs_scene_mt() {
        use crate::intersectable::Intersectable;
        use crate::ray::Ray;
        use crate::sphere::Sphere;
        use crate::threadpool::ThreadPool;
        use nalgebra_glm as glm;
        use rand::prelude::*;

        fn rng_scaled(rng: &mut ThreadRng, scale_factor: f64) -> f64 {
            return (rng.gen::<f64>() - 0.5) * 2.0 * scale_factor;
        }

        fn random_vec3(rng: &mut ThreadRng, scale_factor: f64) -> glm::DVec3 {
            let x = rng_scaled(rng, scale_factor);
            let y = rng_scaled(rng, scale_factor);
            let z = rng_scaled(rng, scale_factor);
            return glm::vec3(x, y, z);
        }

        let ray = Ray::new(glm::vec3(0.0, 0.0, 0.0), glm::vec3(0.0, 0.0, -1.0));
        let mut scene: Vec<Box<dyn Intersectable + Send + Sync>> =
            vec![Box::new(Sphere::new(glm::vec3(0.0, 0.0, -2.0), 1.5))];
        let mut rng = rand::thread_rng();
        let num_objects = 1;
        let scale_factor = 5.0;
        for _ in 0..num_objects {
            scene.push(Box::new(Sphere::new(
                random_vec3(&mut rng, scale_factor),
                rng_scaled(&mut rng, scale_factor),
            )));
        }
        let (t_min, t_max) = (0.01, 1000.0);
        let pool = ThreadPool::new(12);

        let mut chunk_size = scene.len() / pool.get_num_threads();
        if chunk_size == 0 {
            chunk_size = 1;
        }

        // for objects in scene.chunks(chunk_size) {
        //     objects.iter().for_each(|object| {
        //         object.hit(&ray, t_min, t_max);
        //     })
        // }

        pool.scoped(|scope| {
            for objects in scene.chunks(chunk_size) {
                scope.execute(|| {
                    objects.iter().for_each(|object| {
                        object.hit(&ray, t_min, t_max);
                    })
                });
            }
        });
    }
}
