use crate::math::Vec3;
use crate::ray::Ray;

pub struct Camera {
    origin: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    camera_plane_center: Vec3,
}

impl Camera {
    pub fn new(viewport_height: f64, aspect_ratio: f64, focal_length: f64, origin: Vec3) -> Camera {
        let viewport_width = viewport_height as f64 * aspect_ratio;
        let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
        let vertical = Vec3::new(0.0, viewport_height, 0.0);
        let camera_plane_center = &origin - Vec3::new(0.0, 0.0, focal_length);

        return Camera {
            origin,
            horizontal,
            vertical,
            camera_plane_center,
        };
    }

    pub fn get_origin(&self) -> &Vec3 {
        return &self.origin;
    }

    pub fn get_horizontal(&self) -> &Vec3 {
        return &self.horizontal;
    }

    pub fn get_vertical(&self) -> &Vec3 {
        return &self.vertical;
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        return Ray::new(
            self.origin,
            &self.camera_plane_center + u * &self.horizontal + v * &self.vertical - &self.origin,
        );
    }
}
