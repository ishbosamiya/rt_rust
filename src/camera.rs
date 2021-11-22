use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    glm,
    path_trace::ray::Ray,
    rasterize::{
        drawable::Drawable,
        gpu_immediate::{GPUImmediate, GPUPrimType, GPUVertCompType, GPUVertFetchMode},
        gpu_utils, shader,
        texture::TextureRGBAFloat,
    },
    util,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
        *self = Self::from_width(width, self.get_aspect_ratio());
    }

    /// Change sensor's height while keeping aspect ratio the same
    pub fn change_height(&mut self, height: f64) {
        *self = Self::from_height(height, self.get_aspect_ratio());
    }

    /// Change sensor's aspect ratio while keeping sensor width
    /// constant. Reflects the aspect ratio change through the
    /// sensor's height
    pub fn change_aspect_ratio(&mut self, aspect_ratio: f64) {
        *self = Self::from_width(self.get_width(), aspect_ratio);
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
    pub fn get_raycast_direction_uv(&self, uv: &glm::DVec2) -> Option<glm::DVec3> {
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

    /// Get ray given the UVs on the camera sensor.
    ///
    /// See [`Camera::get_raycast_direction_uv()`] for more details,
    /// this function only makes it easy to get the [`Ray`] directly.
    pub fn get_ray(&self, uv: &glm::DVec2) -> Option<Ray> {
        Some(Ray::new(
            self.get_position(),
            self.get_raycast_direction_uv(uv)?,
        ))
    }

    /// Set the camera's position.
    pub fn set_position(&mut self, position: glm::DVec3) {
        self.position = position;
    }

    /// Set the camera's focal length
    ///
    /// # Panics
    ///
    /// Panics if camera sensor is not set.
    pub fn set_focal_length(&mut self, focal_length: f64) {
        self.fov = util::focal_length_to_fov(focal_length, self.get_sensor().unwrap().get_height())
            .to_degrees();
    }

    pub fn set_yaw(&mut self, yaw: f64) {
        self.yaw = yaw;
        self.update_camera_vectors();
    }

    pub fn set_pitch(&mut self, pitch: f64) {
        self.pitch = pitch;
        self.update_camera_vectors();
    }

    pub fn set_yaw_and_pitch(&mut self, yaw: f64, pitch: f64) {
        self.yaw = yaw;
        self.pitch = pitch;
        self.update_camera_vectors();
    }
}

pub struct CameraDrawData {
    imm: Rc<RefCell<GPUImmediate>>,
    image: Option<Rc<RefCell<TextureRGBAFloat>>>,
    alpha_value: f64,
    use_depth_for_image: bool,
}

impl CameraDrawData {
    pub fn new(
        imm: Rc<RefCell<GPUImmediate>>,
        image: Option<Rc<RefCell<TextureRGBAFloat>>>,
        alpha_value: f64,
        use_depth_for_image: bool,
    ) -> Self {
        Self {
            imm,
            image,
            alpha_value,
            use_depth_for_image,
        }
    }
}

impl Drawable for Camera {
    type ExtraData = CameraDrawData;

    type Error = ();

    fn draw(&self, extra_data: &mut Self::ExtraData) -> Result<(), Self::Error> {
        let sensor = self.get_sensor().ok_or(())?;

        // Scale the camera so that the sensor width or height is 1m,
        // the other side is dependent on aspect ratio. So the sensor
        // shown (camera plane) is a constant size and the focal
        // length changes to convey the required information.
        //
        // A camera with a sensor size (width) of 36mm and a focal
        // length of 36mm will be 1m long and 1m wide in 3D space.
        let focal_length = self
            .get_focal_length()
            .expect("by this point focal length should always be available");
        // Equivalent focal length if the sensor was a 36mm sensor
        // (crop factor correction).
        let focal_length = focal_length * 36.0 / sensor.get_width();
        // Focal length required in 3D space, for a focal length of
        // 36mm it is 1m.
        let focal_length = focal_length / 36.0;
        let camera_plane_center = self.position + self.front * focal_length;

        // Sensor width of 1m.
        let horizontal = self.right / 2.0;
        // Sensor height dependent on sensor width.
        let vertical = self.up / 2.0 / sensor.get_aspect_ratio();

        let camera_plane_top_left: glm::Vec3 =
            glm::convert(camera_plane_center + -1.0 * horizontal + 1.0 * vertical);
        let camera_plane_top_right: glm::Vec3 =
            glm::convert(camera_plane_center + 1.0 * horizontal + 1.0 * vertical);
        let camera_plane_bottom_left: glm::Vec3 =
            glm::convert(camera_plane_center + -1.0 * horizontal + -1.0 * vertical);
        let camera_plane_bottom_right: glm::Vec3 =
            glm::convert(camera_plane_center + 1.0 * horizontal + -1.0 * vertical);
        let origin: glm::Vec3 = glm::convert(self.get_position());
        let vertical: glm::Vec3 = glm::convert(vertical);

        let imm = &mut extra_data.imm.borrow_mut();
        let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
            .as_ref()
            .unwrap();
        let color: glm::Vec4 = glm::vec4(0.0, 0.0, 0.0, 1.0);
        smooth_color_3d_shader.use_shader();
        smooth_color_3d_shader.set_mat4("model\0", &glm::identity());

        let format = imm.get_cleared_vertex_format();
        let pos_attr = format.add_attribute(
            "in_pos\0".to_string(),
            GPUVertCompType::F32,
            3,
            GPUVertFetchMode::Float,
        );
        let color_attr = format.add_attribute(
            "in_color\0".to_string(),
            GPUVertCompType::F32,
            4,
            GPUVertFetchMode::Float,
        );

        imm.begin(GPUPrimType::Lines, 16, smooth_color_3d_shader);

        // from origin to the plane
        draw_line(
            imm,
            &origin,
            &camera_plane_top_left,
            pos_attr,
            color_attr,
            &color,
        );
        draw_line(
            imm,
            &origin,
            &camera_plane_top_right,
            pos_attr,
            color_attr,
            &color,
        );
        draw_line(
            imm,
            &origin,
            &camera_plane_bottom_left,
            pos_attr,
            color_attr,
            &color,
        );
        draw_line(
            imm,
            &origin,
            &camera_plane_bottom_right,
            pos_attr,
            color_attr,
            &color,
        );

        // the plane
        draw_line(
            imm,
            &camera_plane_top_left,
            &camera_plane_top_right,
            pos_attr,
            color_attr,
            &color,
        );
        draw_line(
            imm,
            &camera_plane_top_right,
            &camera_plane_bottom_right,
            pos_attr,
            color_attr,
            &color,
        );
        draw_line(
            imm,
            &camera_plane_bottom_right,
            &camera_plane_bottom_left,
            pos_attr,
            color_attr,
            &color,
        );
        draw_line(
            imm,
            &camera_plane_bottom_left,
            &camera_plane_top_left,
            pos_attr,
            color_attr,
            &color,
        );

        imm.end();

        // triangle at the top
        imm.begin(GPUPrimType::Tris, 3, smooth_color_3d_shader);

        draw_triangle(
            imm,
            &camera_plane_top_left,
            &camera_plane_top_right,
            &((camera_plane_top_left + camera_plane_top_right) / 2.0 + vertical),
            pos_attr,
            color_attr,
            &color,
        );

        imm.end();

        // draw image in the camera plane
        if let Some(image) = &extra_data.image {
            if !extra_data.use_depth_for_image {
                unsafe {
                    gl::Disable(gl::DEPTH_TEST);
                }
            }

            let scale_x = (camera_plane_top_left - camera_plane_top_right).norm() as _;
            let scale_z = (camera_plane_top_left - camera_plane_bottom_left).norm() as _;
            gpu_utils::draw_plane_with_image(
                &camera_plane_center,
                &glm::vec3(scale_x, 1.0, scale_z),
                &(camera_plane_center - self.get_position()).normalize(),
                &mut image.borrow_mut(),
                extra_data.alpha_value,
                imm,
            );

            if !extra_data.use_depth_for_image {
                unsafe {
                    gl::Enable(gl::DEPTH_TEST);
                }
            }
        }

        Ok(())
    }
}

fn draw_line(
    imm: &mut GPUImmediate,
    p1: &glm::Vec3,
    p2: &glm::Vec3,
    pos_attr: usize,
    color_attr: usize,
    color: &glm::Vec4,
) {
    imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
    imm.vertex_3f(pos_attr, p1[0], p1[1], p1[2]);
    imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
    imm.vertex_3f(pos_attr, p2[0], p2[1], p2[2]);
}

fn draw_triangle(
    imm: &mut GPUImmediate,
    p1: &glm::Vec3,
    p2: &glm::Vec3,
    p3: &glm::Vec3,
    pos_attr: usize,
    color_attr: usize,
    color: &glm::Vec4,
) {
    imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
    imm.vertex_3f(pos_attr, p1[0], p1[1], p1[2]);
    imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
    imm.vertex_3f(pos_attr, p2[0], p2[1], p2[2]);
    imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
    imm.vertex_3f(pos_attr, p3[0], p3[1], p3[2]);
}
