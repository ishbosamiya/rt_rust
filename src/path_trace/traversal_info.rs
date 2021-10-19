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
}

impl SingleRayInfo {
    /// Creates new `SingleRayInfo`
    pub fn new(ray: Ray, co: Option<glm::DVec3>, color: glm::DVec3) -> Self {
        Self { ray, co, color }
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
}

pub struct TraversalInfo {
    traversal: Vec<SingleRayInfo>,
}

impl TraversalInfo {
    pub fn new() -> Self {
        Self {
            traversal: Vec::new(),
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
}

impl Default for TraversalInfo {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TraversalInfoDrawData {
    imm: Rc<RefCell<GPUImmediate>>,
}

impl TraversalInfoDrawData {
    pub fn new(imm: Rc<RefCell<GPUImmediate>>) -> Self {
        Self { imm }
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

        imm.begin(
            GPUPrimType::Lines,
            self.get_traversal().len() * 2,
            smooth_color_3d_shader,
        );

        self.get_traversal().iter().for_each(|info| {
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
        });

        imm.end();

        Ok(())
    }
}
