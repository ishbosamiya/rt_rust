use rt::camera::Camera;
use rt::image::{Image, PPM};
use rt::intersectable::Intersectable;
use rt::math::{Scalar, Vec3};
use rt::ray::Ray;
use rt::sphere::Sphere;

use nalgebra_glm as glm;

fn main() {
    let width = 1280;
    let height = 720;
    let mut image = Image::new(width, height);

    let viewport_height = 2.0;
    let aspect_ratio = width as f64 / height as f64;
    let focal_length = 1.0;
    let origin = glm::vec3(0.0, 0.0, 0.0);
    let camera = Camera::new(viewport_height, aspect_ratio, focal_length, origin);

    let scene: Vec<Box<dyn Intersectable>> =
        vec![Box::new(Sphere::new(glm::vec3(0.0, 0.0, -2.0), 1.5))];

    for (j, row) in image.get_pixels_mut().iter_mut().enumerate() {
        for (i, pixel) in row.iter_mut().enumerate() {
            let j = height - j - 1;

            // use opengl coords, (0.0, 0.0) is center; (1.0, 1.0) is
            // top right; (-1.0, -1.0) is bottom left
            let u = ((i as Scalar / (width - 1) as Scalar) - 0.5) * 2.0;
            let v = ((j as Scalar / (height - 1) as Scalar) - 0.5) * 2.0;

            let ray = camera.get_ray(u, v);

            *pixel = trace_ray(&ray, &camera, &scene);
        }
    }

    let ppm = PPM::new(&image);
    ppm.write_to_file("image.ppm").unwrap();
}

fn get_background_color(ray: &Ray, camera: &Camera) -> Vec3 {
    let color_1 = glm::vec3(0.8, 0.8, 0.8);
    let color_2 = glm::vec3(0.2, 0.2, 0.8);

    let camera_origin_y = camera.get_origin()[1];
    let camera_vertical_range = camera.get_vertical()[1];
    let y_val = (camera_origin_y + ray.get_direction()[1]) / camera_vertical_range;
    let y_val = (y_val + 1.0) / 2.0;

    return glm::lerp(&color_1, &color_2, y_val);
}

// x: current point
// x_prime: previous point
// x_prime_prime: previous's previous point
// g: geometry term, 1/(r^2) where r is distance of x_prime to x
// e: intensity of emitted light by x_prime reaching x
// i: intensity of light from x_prime to x
// p: intensity of light scattered from x_prime_prime to x by a patch on surface at x_prime
fn trace_ray(ray: &Ray, camera: &Camera, scene: &Vec<Box<dyn Intersectable>>) -> Vec3 {
    let mut val = glm::zero();
    scene.iter().for_each(|object| {
        if let Some(_) = object.hit(ray, 0.01, 1000.0) {
            val = glm::vec3(1.0, 1.0, 0.0);
        } else {
            val = get_background_color(ray, camera);
        }
    });
    return val;
}
