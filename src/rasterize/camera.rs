use crate::glm;

pub struct Camera {
    position: glm::DVec3,
    front: glm::DVec3,
    up: glm::DVec3,
    right: glm::DVec3,
    world_up: glm::DVec3,
    yaw: f64,
    pitch: f64,
    zoom: f64,
    near_plane: f64,
    far_plane: f64,
}

impl Camera {
    pub fn new(position: glm::DVec3, up: glm::DVec3, yaw: f64, pitch: f64, zoom: f64) -> Camera {
        let mut camera = Camera {
            position,
            yaw,
            pitch,
            world_up: up,
            front: glm::vec3(0.0, 0.0, -1.0),
            right: glm::zero(),
            up,
            zoom,
            near_plane: 0.1,
            far_plane: 1000.0,
        };

        camera.update_camera_vectors();

        camera
    }

    fn update_camera_vectors(&mut self) {
        let yaw_radians = f64::to_radians(self.yaw);
        let pitch_radians = f64::to_radians(self.pitch);
        let front: glm::DVec3 = glm::vec3(
            yaw_radians.cos() * pitch_radians.cos(),
            pitch_radians.sin(),
            yaw_radians.sin() * pitch_radians.cos(),
        );
        self.front = glm::normalize(&front);

        self.right = glm::normalize(&glm::cross(&front, &self.world_up));
        self.up = glm::normalize(&glm::cross(&self.right, &front));
    }

    pub fn get_world_up(&self) -> &glm::DVec3 {
        &self.world_up
    }

    pub fn get_position(&self) -> glm::DVec3 {
        self.position
    }

    pub fn get_front(&self) -> glm::DVec3 {
        self.front
    }

    pub fn get_right(&self) -> glm::DVec3 {
        self.right
    }

    pub fn get_up(&self) -> glm::DVec3 {
        self.up
    }

    pub fn get_near_plane(&self) -> f64 {
        self.near_plane
    }

    pub fn get_far_plane(&self) -> f64 {
        self.far_plane
    }

    pub fn get_yaw(&self) -> f64 {
        self.yaw
    }

    pub fn get_pitch(&self) -> f64 {
        self.pitch
    }

    pub fn get_zoom(&self) -> f64 {
        self.zoom
    }

    pub fn get_view_matrix(&self) -> glm::DMat4 {
        glm::look_at(&self.position, &(self.position + self.front), &self.up)
    }

    pub fn get_projection_matrix(&self, width: usize, height: usize) -> glm::DMat4 {
        glm::perspective(
            width as f64 / height as f64,
            self.zoom.to_radians(),
            self.near_plane,
            self.far_plane,
        )
    }

    pub fn get_ortho_matrix(&self, width: usize, height: usize) -> glm::DMat4 {
        glm::ortho(
            0.0,
            width as f64,
            0.0,
            height as f64,
            self.near_plane,
            self.far_plane,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn pan(
        &mut self,
        mouse_start_x: f64,
        mouse_start_y: f64,
        mouse_end_x: f64,
        mouse_end_y: f64,
        len: f64,
        width: usize,
        height: usize,
    ) {
        if (mouse_start_x - mouse_end_x).abs() < f64::EPSILON
            && (mouse_start_y - mouse_end_y).abs() < f64::EPSILON
        {
            return;
        }
        let clip_x = mouse_start_x * 2.0 / width as f64 - 1.0;
        let clip_y = 1.0 - mouse_start_y * 2.0 / height as f64;

        let clip_end_x = mouse_end_x * 2.0 / width as f64 - 1.0;
        let clip_end_y = 1.0 - mouse_end_y * 2.0 / height as f64;

        let inverse_mvp =
            glm::inverse(&(self.get_projection_matrix(width, height) * self.get_view_matrix()));
        let out_vector = inverse_mvp * glm::vec4(clip_x, clip_y, 0.0, 1.0);
        let world_pos = glm::vec3(
            out_vector.x / out_vector.w,
            out_vector.y / out_vector.w,
            out_vector.z / out_vector.w,
        );

        let out_end_vector = inverse_mvp * glm::vec4(clip_end_x, clip_end_y, 0.0, 1.0);
        let world_pos_2 = glm::vec3(
            out_end_vector.x / out_end_vector.w,
            out_end_vector.y / out_end_vector.w,
            out_end_vector.z / out_end_vector.w,
        );

        let dir = world_pos_2 - world_pos;

        let offset = glm::length(&dir) * glm::normalize(&dir) * self.zoom * len / 2.0;

        self.position -= offset;
    }

    pub fn rotate_wrt_camera_origin(
        &mut self,
        mouse_start_x: f64,
        mouse_start_y: f64,
        mouse_end_x: f64,
        mouse_end_y: f64,
        mouse_sensitivity: f64,
        constrain_pitch: bool,
    ) {
        let x_offset = (mouse_end_x - mouse_start_x) * mouse_sensitivity;
        let y_offset = (mouse_start_y - mouse_end_y) * mouse_sensitivity;

        self.yaw += x_offset;
        self.pitch += y_offset;

        if constrain_pitch {
            if self.pitch > 89.0 {
                self.pitch = 89.0;
            } else if self.pitch < -89.0 {
                self.pitch = -89.0;
            }
        }

        self.update_camera_vectors();
    }

    pub fn move_forward(&mut self, mouse_start_y: f64, mouse_end_y: f64, height: usize) {
        let clip_y = 1.0 - mouse_start_y * 2.0 / height as f64;
        let clip_end_y = 1.0 - mouse_end_y * 2.0 / height as f64;

        let move_by = clip_end_y - clip_y;

        self.position += self.front * move_by;
    }

    pub fn zoom(&mut self, scroll_y: f64) {
        let min = 1.0;
        let max = 90.0;
        if self.zoom >= min && self.zoom <= max {
            self.zoom -= scroll_y;
        }
        if self.zoom < min {
            self.zoom = min;
        }
        if self.zoom > max {
            self.zoom = max;
        }
    }

    pub fn get_raycast_direction(
        &self,
        mouse_x: f64,
        mouse_y: f64,
        width: usize,
        height: usize,
    ) -> glm::DVec3 {
        let x = (2.0 * mouse_x) / width as f64 - 1.0;
        let y = 1.0 - (2.0 * mouse_y) / height as f64;

        let ray_clip = glm::vec4(x, y, -1.0, 1.0);

        let ray_eye = glm::inverse(&self.get_projection_matrix(width, height)) * ray_clip;
        let ray_eye = glm::vec4(ray_eye[0], ray_eye[1], -1.0, 0.0);

        let ray_wor = glm::inverse(&self.get_view_matrix()) * ray_eye;

        glm::normalize(&glm::vec4_to_vec3(&ray_wor))
    }
}
