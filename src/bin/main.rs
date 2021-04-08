use rt::camera::Camera;
use rt::image::{Image, PPM};
use rt::math::{Scalar, Vec3};
use rt::ray::Ray;

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

    for (j, row) in image.get_pixels_mut().iter_mut().enumerate() {
        for (i, pixel) in row.iter_mut().enumerate() {
            let j = height - j - 1;

            // use opengl coords, (0.0, 0.0) is center; (1.0, 1.0) is
            // top right; (-1.0, -1.0) is bottom left
            let u = ((i as Scalar / (width - 1) as Scalar) - 0.5) * 2.0;
            let v = ((j as Scalar / (height - 1) as Scalar) - 0.5) * 2.0;

            let ray = camera.get_ray(u, v);

            let background_color = get_background_color(&ray, &camera);

            *pixel = background_color;
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
