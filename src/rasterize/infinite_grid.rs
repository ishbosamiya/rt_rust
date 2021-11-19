use std::{cell::RefCell, rc::Rc};

use crate::{
    glm,
    rasterize::{
        drawable::Drawable,
        gpu_immediate::{GPUImmediate, GPUPrimType, GPUVertCompType, GPUVertFetchMode},
        shader,
    },
};

/// Generates an infinite grid in the xz plane through some shader
/// magic
///
/// See
/// <https://github.com/martin-pr/possumwood/wiki/Infinite-ground-plane-using-GLSL-shaders>
/// and
/// <https://asliceofrendering.com/scene%20helper/2020/01/05/InfiniteGrid/>
/// for more details about this approach
///
/// Drawing the grid requires blending, so it turns it on using
/// gl::Enable(gl::BLEND); but doesn't not disable it since even the
/// blend function is set so caller anyway has to reset things if
/// needed
pub struct InfiniteGrid {
    plane_vert_positions: Vec<glm::Vec3>,
}

impl InfiniteGrid {
    pub fn new() -> Self {
        Self {
            plane_vert_positions: vec![
                glm::vec3(1.0, 1.0, 0.0),
                glm::vec3(-1.0, -1.0, 0.0),
                glm::vec3(-1.0, 1.0, 0.0),
                glm::vec3(-1.0, -1.0, 0.0),
                glm::vec3(1.0, 1.0, 0.0),
                glm::vec3(1.0, -1.0, 0.0),
            ],
        }
    }
}

impl Default for InfiniteGrid {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InfiniteGridDrawData {
    imm: Rc<RefCell<GPUImmediate>>,
    color: glm::DVec4,
}

impl InfiniteGridDrawData {
    pub fn new(imm: Rc<RefCell<GPUImmediate>>, color: glm::DVec4) -> Self {
        Self { imm, color }
    }
}

impl Drawable for InfiniteGrid {
    type ExtraData = InfiniteGridDrawData;
    type Error = ();

    fn draw(&self, extra_data: &mut InfiniteGridDrawData) -> Result<(), ()> {
        let mut imm = extra_data.imm.borrow_mut();

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        let infinite_grid_shader = shader::builtins::get_infinite_grid_shader()
            .as_ref()
            .unwrap();

        infinite_grid_shader.use_shader();

        infinite_grid_shader.set_vec4("grid_color\0", &glm::convert(extra_data.color));

        let format = imm.get_cleared_vertex_format();
        let pos_attr = format.add_attribute(
            "in_pos\0".to_string(),
            GPUVertCompType::F32,
            3,
            GPUVertFetchMode::Float,
        );

        imm.begin(GPUPrimType::Tris, 6, infinite_grid_shader);

        self.plane_vert_positions.iter().for_each(|pos| {
            imm.vertex_3f(pos_attr, pos[0], pos[1], pos[2]);
        });

        imm.end();

        Ok(())
    }

    fn draw_wireframe(&self, _extra_data: &mut InfiniteGridDrawData) -> Result<(), ()> {
        unreachable!()
    }
}
