pub mod blinn;
pub mod bsdf;
pub mod bvh;
pub mod camera;
pub mod drawable;
pub mod fps;
pub mod gl_camera;
pub mod gpu_immediate;
pub mod gpu_utils;
pub mod image;
pub mod intersectable;
pub mod math;
pub mod mesh;
pub mod meshio;
pub mod ray;
pub mod scene;
pub mod shader;
pub mod sphere;
pub mod texture;
pub mod util;
pub mod blinnphong;

pub use nalgebra_glm as glm;

use crate::bsdf::BSDFTemplate;
use crate::camera::Camera;
use crate::intersectable::Intersectable;
use crate::ray::Ray;
use crate::scene::Scene;

fn get_background_color(ray: &Ray, camera: &Camera) -> glm::DVec3 {
    let color_1 = glm::vec3(0.8, 0.8, 0.8);
    let color_2 = glm::vec3(0.2, 0.2, 0.8);

    let camera_origin_y = camera.get_origin()[1];
    let camera_vertical_range = camera.get_vertical()[1];
    let y_val = (camera_origin_y + ray.get_direction()[1]) / camera_vertical_range;
    let y_val = (y_val + 1.0) / 2.0;

    glm::lerp(&color_1, &color_2, y_val)
}

// x: current point
// x_prime: previous point
// x_prime_prime: previous's previous point
// g: geometry term, 1/(r^2) where r is distance of x_prime to x
// e: intensity of emitted light by x_prime reaching x
// i: intensity of light from x_prime to x
// p: intensity of light scattered from x_prime_prime to x by a patch on surface at x_prime
pub fn trace_ray(ray: &Ray, camera: &Camera, scene: &'static Scene, depth: usize) -> glm::DVec3 {
    if depth == 0 {
        return glm::zero();
    }
    let val;
    if let Some(info) = scene.hit(ray, 0.01, 1000.0) {
        // Creating bsdf template and calling function
        // Random values as of now
        let template = BSDFTemplate {
            roughness: 0.01_f64,
            brightness: 10.0_f64,
            opacity: 1.0_f64,
        };
        // diffuse shader
        // Shader code : TODO Check if it works with bsdf
        // Modified BSDF Code
<<<<<<< HEAD
        let target = info.get_point() + template.setup(ray.get_direction(), &info.get_point());
=======
         let target = template.setup(ray.get_direction(), &info.get_point());
>>>>>>> d501be01c9135ac98d10371a7d4943372b2ef425
        // Original code
        //let target = info.get_point() + info.get_normal().unwrap() + math::random_in_unit_sphere();
        val = 0.5
            * trace_ray(
                &Ray::new(*info.get_point(), target - info.get_point()),
                camera,
                scene,
                depth - 1,
            );
    } else {
        val = get_background_color(ray, camera);
    }
    val
}

#[cfg(test)]
mod tests {
    #[test]
    fn single_ray_vs_scene_mt() {
        use crate::glm;
        use crate::intersectable::Intersectable;
        use crate::ray::Ray;
        use crate::sphere::Sphere;
        use rand::prelude::*;

        fn rng_scaled(rng: &mut ThreadRng, scale_factor: f64) -> f64 {
            (rng.gen::<f64>() - 0.5) * 2.0 * scale_factor
        }

        fn random_vec3(rng: &mut ThreadRng, scale_factor: f64) -> glm::DVec3 {
            let x = rng_scaled(rng, scale_factor);
            let y = rng_scaled(rng, scale_factor);
            let z = rng_scaled(rng, scale_factor);
            glm::vec3(x, y, z)
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

        let mut chunk_size = scene.len();
        if chunk_size == 0 {
            chunk_size = 1;
        }

        for objects in scene.chunks(chunk_size) {
            objects.iter().for_each(|object| {
                object.hit(&ray, t_min, t_max);
            })
        }
    }
}
