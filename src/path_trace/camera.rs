use std::{cell::RefCell, rc::Rc};

use super::ray::Ray;
use crate::{
    glm,
    rasterize::{
        drawable::Drawable,
        gpu_immediate::{GPUImmediate, GPUPrimType, GPUVertCompType, GPUVertFetchMode},
        gpu_utils::draw_plane_with_image,
        shader,
        texture::TextureRGBAFloat,
    },
};

#[derive(Debug, Clone)]
pub struct Camera {
    origin: glm::DVec3,
    horizontal: glm::DVec3,
    vertical: glm::DVec3,
    camera_plane_center: glm::DVec3,
}

impl Camera {
    pub fn new(
        sensor_height: f64,
        aspect_ratio: f64,
        focal_length: f64,
        origin: glm::DVec3,
    ) -> Camera {
        // The sensor height and width get doubled since the UV's used
        // are OpenGL based so they range from -1 to 1. To fix the
        // doubling, the sensor height must be halved before
        // calculating horizontal and vertical.
        let sensor_height = sensor_height / 2.0;

        let sensor_width = sensor_height as f64 * aspect_ratio;
        let horizontal = glm::vec3(sensor_width, 0.0, 0.0);
        let vertical = glm::vec3(0.0, sensor_height, 0.0);
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

    /// Get a reference to the camera's camera plane center.
    pub fn get_camera_plane_center(&self) -> &glm::DVec3 {
        &self.camera_plane_center
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray::new(
            self.origin,
            self.camera_plane_center + u * self.horizontal + v * self.vertical - self.origin,
        )
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
        let camera_plane_top_left: glm::Vec3 =
            glm::convert(self.camera_plane_center + -1.0 * self.horizontal + 1.0 * self.vertical);
        let camera_plane_top_right: glm::Vec3 =
            glm::convert(self.camera_plane_center + 1.0 * self.horizontal + 1.0 * self.vertical);
        let camera_plane_bottom_left: glm::Vec3 =
            glm::convert(self.camera_plane_center + -1.0 * self.horizontal + -1.0 * self.vertical);
        let camera_plane_bottom_right: glm::Vec3 =
            glm::convert(self.camera_plane_center + 1.0 * self.horizontal + -1.0 * self.vertical);
        let origin: glm::Vec3 = glm::convert(self.origin);
        let vertical: glm::Vec3 = glm::convert(self.vertical);

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
            draw_plane_with_image(
                &self.camera_plane_center,
                &glm::vec3(scale_x, 1.0, scale_z),
                &-(self.camera_plane_center - self.origin).normalize(),
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
