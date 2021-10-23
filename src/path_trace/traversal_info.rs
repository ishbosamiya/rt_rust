use std::{cell::RefCell, rc::Rc};

use crate::{
    glm,
    rasterize::{
        drawable::Drawable,
        gpu_immediate::{GPUImmediate, GPUPrimType, GPUVertCompType, GPUVertFetchMode},
        shader,
    },
};

use super::ray::Ray;

pub struct SingleRayInfo {
    /// the ray for which the info is defined
    ray: Ray,
    /// the point of intersection for the ray
    ///
    /// it is possible for the ray to be shaded by the environment so
    /// `co` may not exist, thus is made optional
    co: Option<glm::DVec3>,
    /// color/intensity of light of the ray
    color: glm::DVec3,
    /// normal at co if co exists
    normal: Option<glm::DVec3>,
}

impl SingleRayInfo {
    /// Creates new `SingleRayInfo`
    pub fn new(
        ray: Ray,
        co: Option<glm::DVec3>,
        color: glm::DVec3,
        normal: Option<glm::DVec3>,
    ) -> Self {
        Self {
            ray,
            co,
            color,
            normal,
        }
    }

    /// Get a reference to the single ray info's color.
    pub fn get_ray(&self) -> &Ray {
        &self.ray
    }

    /// Get a reference to the single ray info's co.
    pub fn get_co(&self) -> &Option<glm::DVec3> {
        &self.co
    }

    /// Get a reference to the single ray info's color.
    pub fn get_color(&self) -> &glm::DVec3 {
        &self.color
    }

    /// Get a reference to the single ray info's normal.
    pub fn get_normal(&self) -> &Option<glm::DVec3> {
        &self.normal
    }
}

pub struct TraversalInfo {
    traversal: Vec<SingleRayInfo>,
    min_bounces: usize,
    max_bounces: usize,
}

impl TraversalInfo {
    pub fn new() -> Self {
        Self {
            traversal: Vec::new(),
            min_bounces: 0,
            max_bounces: 100,
        }
    }

    pub fn get_traversal(&self) -> &[SingleRayInfo] {
        self.traversal.as_slice()
    }

    pub fn add_ray(&mut self, info: SingleRayInfo) {
        // TODO(ish): add some assertions to ensure that the traversal
        // path can form a continuous path
        self.traversal.push(info);
    }

    pub fn get_bounces(&self) -> std::ops::Range<usize> {
        self.min_bounces..self.max_bounces
    }

    pub fn set_bounce_range(&mut self, start: usize, end: usize) {
        self.min_bounces = start;
        self.max_bounces = end;
    }
}

impl Default for TraversalInfo {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TraversalInfoDrawData {
    imm: Rc<RefCell<GPUImmediate>>,
    draw_normal_at_hit_points: bool,
    normals_size: f64,
    normals_color: glm::DVec4,
}

impl TraversalInfoDrawData {
    pub fn new(
        imm: Rc<RefCell<GPUImmediate>>,
        draw_normal_at_hit_points: bool,
        normals_size: f64,
        normals_color: glm::DVec4,
    ) -> Self {
        Self {
            imm,
            draw_normal_at_hit_points,
            normals_size,
            normals_color,
        }
    }
}

impl Drawable for TraversalInfo {
    type ExtraData = TraversalInfoDrawData;

    type Error = ();

    fn draw(&self, extra_data: &mut Self::ExtraData) -> Result<(), Self::Error> {
        let mut imm = extra_data.imm.borrow_mut();

        let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
            .as_ref()
            .unwrap();
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

        imm.begin_at_most(
            GPUPrimType::Lines,
            self.get_traversal().len() * 2,
            smooth_color_3d_shader,
        );

        let bounce_range = self.get_bounces();
        let min_bounces = bounce_range.start;
        let max_bounces = bounce_range.end;
        self.get_traversal()
            .iter()
            .rev()
            .enumerate()
            .skip(min_bounces - 1)
            .try_for_each(|(index, info)| {
                if index == max_bounces {
                    None
                } else {
                    let p1: glm::Vec3 = glm::convert(*info.get_ray().get_origin());
                    let p2 = if let Some(co) = info.get_co() {
                        *co
                    } else {
                        info.get_ray().at(1000.0)
                    };
                    let p2: glm::Vec3 = glm::convert(p2);
                    let color: glm::Vec3 = glm::convert(*info.get_color());

                    imm.attr_4f(color_attr, color[0], color[1], color[2], 1.0);
                    imm.vertex_3f(pos_attr, p1[0], p1[1], p1[2]);

                    imm.attr_4f(color_attr, color[0], color[1], color[2], 1.0);
                    imm.vertex_3f(pos_attr, p2[0], p2[1], p2[2]);

                    Some(())
                }
            });

        imm.end();

        if extra_data.draw_normal_at_hit_points {
            imm.begin_at_most(
                GPUPrimType::Lines,
                self.get_traversal().len() * 2,
                smooth_color_3d_shader,
            );

            self.get_traversal()
                .iter()
                .rev()
                .enumerate()
                .skip(min_bounces - 1)
                .try_for_each(|(index, info)| {
                    if index == max_bounces {
                        None
                    } else {
                        let p1 = if let Some(co) = info.get_co() {
                            *co
                        } else {
                            return None;
                        };
                        let p2 =
                            p1 + (info.get_normal().unwrap().normalize() * extra_data.normals_size);
                        let p1: glm::Vec3 = glm::convert(p1);
                        let p2: glm::Vec3 = glm::convert(p2);
                        let color: glm::Vec4 = glm::convert(extra_data.normals_color);

                        imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
                        imm.vertex_3f(pos_attr, p1[0], p1[1], p1[2]);

                        imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
                        imm.vertex_3f(pos_attr, p2[0], p2[1], p2[2]);
                        Some(())
                    }
                });

            imm.end();
        }

        Ok(())
    }
}
