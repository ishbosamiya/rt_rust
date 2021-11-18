use crate::{glm, util};

#[derive(Debug)]
pub struct Camera {
    /// position of the camera in 3D space
    position: glm::DVec3,
    /// front direction of the camera
    front: glm::DVec3,
    /// up direction of the camera
    up: glm::DVec3,
    /// right direction of the camera
    right: glm::DVec3,
    /// up direction of the world with respect to which the camera's
    /// front, up and right are defined
    world_up: glm::DVec3,
    /// yaw of the camera
    yaw: f64,
    /// pitch of the camera
    pitch: f64,
    /// vertical field of view of the camera in degrees
    fov: f64,

    /// near clipping plane of the camera
    near_plane: f64,
    /// far clipping plane of the camera
    far_plane: f64,

    /// camera's sensor
    sensor: Option<Sensor>,
}

/// Camera sensor
#[derive(Debug, Clone, Copy)]
pub struct Sensor {
    /// sensor width
    width: f64,
    /// sensor height
    height: f64,
    /// aspect ratio of the sensor, width of the sensor with respect
    /// to the height of the aspect
    aspect_ratio: f64,
}

impl Sensor {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            aspect_ratio: width / height,
        }
    }

    pub fn from_width(width: f64, aspect_ratio: f64) -> Self {
        Self {
            width,
            height: width / aspect_ratio,
            aspect_ratio,
        }
    }

    pub fn from_height(height: f64, aspect_ratio: f64) -> Self {
        Self {
            width: height * aspect_ratio,
            height,
            aspect_ratio,
        }
    }

    /// Get sensor's width.
    pub fn get_width(&self) -> f64 {
        self.width
    }

    /// Get sensor's height.
    pub fn get_height(&self) -> f64 {
        self.height
    }

    /// Get sensor's aspect ratio.
    pub fn get_aspect_ratio(&self) -> f64 {
        self.aspect_ratio
    }

    /// Change sensor's width while keeping aspect ratio the same
    pub fn change_width(&mut self, width: f64) {
        *self = Self::from_width(width, self.get_aspect_ratio())
    }

    /// Change sensor's height while keeping aspect ratio the same
    pub fn change_height(&mut self, height: f64) {
        *self = Self::from_height(height, self.get_aspect_ratio())
    }
}

impl Camera {
    pub fn new(
        position: glm::DVec3,
        up: glm::DVec3,
        yaw: f64,
        pitch: f64,
        fov: f64,
        sensor: Option<Sensor>,
    ) -> Camera {
        let mut camera = Camera {
            position,
            yaw,
            pitch,
            world_up: up,
            front: glm::vec3(0.0, 0.0, -1.0),
            right: glm::zero(),
            up,
            fov,
            near_plane: 0.1,
            far_plane: 1000.0,
            sensor,
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

    pub fn get_fov(&self) -> f64 {
        self.fov
    }

    pub fn get_focal_length(&self) -> Option<f64> {
        Some(util::fov_to_focal_length(
            self.get_fov().to_radians(),
            self.get_sensor()?.get_height(),
        ))
    }

    /// Get reference to camera's sensor
    pub fn get_sensor(&self) -> Option<&Sensor> {
        self.sensor.as_ref()
    }

    /// Get camera's sensor
    pub fn get_sensor_no_ref(&self) -> Option<Sensor> {
        self.sensor
    }

    /// Get mutable reference to camera's sensor
    pub fn get_sensor_mut(&mut self) -> &mut Option<Sensor> {
        &mut self.sensor
    }

    pub fn get_view_matrix(&self) -> glm::DMat4 {
        glm::look_at(&self.position, &(self.position + self.front), &self.up)
    }

    pub fn get_projection_matrix(&self, width: usize, height: usize) -> glm::DMat4 {
        glm::perspective(
            width as f64 / height as f64,
            self.fov.to_radians(),
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

        let offset = glm::length(&dir) * glm::normalize(&dir) * self.fov * len / 2.0;

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
        if self.fov >= min && self.fov <= max {
            self.fov -= scroll_y;
        }
        if self.fov < min {
            self.fov = min;
        }
        if self.fov > max {
            self.fov = max;
        }
    }

    pub fn get_raycast_direction(
        &mut self,
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

    /// Get ray cast direction given the UVs on the camera sensor
    /// through which the ray should pass.
    ///
    /// If no sensor is available in the camera, return None.
    ///
    /// UVs are defined as (0.0, 0.0) at center, (1.0, 1.0) as top
    /// right corner and (-1.0, -1.0) as bottom left corner of the
    /// sensor.
    pub fn get_raycast_direction_uv(&self, uv: glm::DVec2) -> Option<glm::DVec3> {
        let sensor = self.get_sensor()?;

        let camera_plane_center = self.position
            + self.front
                * self
                    .get_focal_length()
                    .expect("by this point focal length should always be available");

        let horizontal = self.right * sensor.get_width() / 2.0;
        let vertical = self.up * sensor.get_height() / 2.0;

        let point_on_sensor = camera_plane_center + uv[0] * horizontal + uv[1] * vertical;

        Some((point_on_sensor - self.position).normalize())
    }
}
