use crate::glm;
use crate::ray::Ray;

pub struct Camera {
    origin: glm::DVec3,
    horizontal: glm::DVec3,
    vertical: glm::DVec3,
    camera_plane_center: glm::DVec3,
}

impl Camera {
    pub fn new(
        viewport_height: f64,
        aspect_ratio: f64,
        focal_length: f64,
        origin: glm::DVec3,
    ) -> Camera {
        let viewport_width = viewport_height as f64 * aspect_ratio;
        let horizontal = glm::vec3(viewport_width, 0.0, 0.0);
        let vertical = glm::vec3(0.0, viewport_height, 0.0);
        let camera_plane_center = origin - glm::vec3(0.0, 0.0, focal_length);

        Camera {
            origin,
            horizontal,
            vertical,
            camera_plane_center,
        }
    }

    pub fn get_origin(&self) -> &glm::DVec3 {
        &self.origin
    }

    pub fn get_horizontal(&self) -> &glm::DVec3 {
        &self.horizontal
    }

    pub fn get_vertical(&self) -> &glm::DVec3 {
        &self.vertical
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray::new(
            self.origin,
            self.camera_plane_center + u * self.horizontal + v * self.vertical - self.origin,
        )
    }
}
