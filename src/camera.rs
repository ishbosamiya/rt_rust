use crate::math::Vec3;
use crate::ray::Ray;

pub struct Camera {
    origin: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    lower_left_corner: Vec3,
}

impl Camera {
    pub fn new(viewport_height: f64, aspect_ratio: f64, focal_length: f64, origin: Vec3) -> Camera {
        let viewport_width = viewport_height as f64 * aspect_ratio;
        let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
        let vertical = Vec3::new(0.0, viewport_height, 0.0);
        let lower_left_corner =
            &origin - &horizontal / 2.0 - &vertical / 2.0 - Vec3::new(0.0, 0.0, focal_length);

        return Camera {
            origin,
            horizontal,
            vertical,
            lower_left_corner,
        };
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        return Ray::new(
            self.origin,
            &self.lower_left_corner + u * &self.horizontal + v * &self.vertical - &self.origin,
        );
    }
}
