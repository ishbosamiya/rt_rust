use std::convert::TryInto;

use memoffset::offset_of;

use crate::glm;

use super::drawable::Drawable;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GLVert {
    pos: glm::Vec3,
    uv: glm::Vec2,
    normal: glm::Vec3,
}

impl GLVert {
    pub fn new(pos: glm::Vec3, uv: glm::Vec2, normal: glm::Vec3) -> Self {
        Self { pos, uv, normal }
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    i1: gl::types::GLuint,
    i2: gl::types::GLuint,
    i3: gl::types::GLuint,
}

impl Triangle {
    pub fn new(i1: gl::types::GLuint, i2: gl::types::GLuint, i3: gl::types::GLuint) -> Self {
        Self { i1, i2, i3 }
    }
}

pub struct GLMesh {
    // no need to store the verts and indices, currently there is no
    // way to update the verts or indices thus does not need to be
    // stored on the CPU as well.
    //
    // verts: Vec<GLVert>,
    // triangles: Vec<Triangle>,
    num_triangles: usize,

    vao: gl::types::GLuint,
    // no need to store the vbo and ebo since there is no need to
    // update it as of right now and thus not required.
    //
    // vbo: gl::types::GLuint,
    // ebo: gl::types::GLuint,
}

impl GLMesh {
    pub fn new(verts: &[GLVert], triangles: &[Triangle]) -> Self {
        Self::setup(verts, triangles)
    }

    fn setup(verts: &[GLVert], triangles: &[Triangle]) -> Self {
        let (vao, vbo, ebo) = unsafe {
            let mut vao: gl::types::GLuint = 0;
            let mut vbo: gl::types::GLuint = 0;
            let mut ebo: gl::types::GLuint = 0;
            // generate the buffers needed
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
            (vao, vbo, ebo)
        };

        if vao == 0 || vbo == 0 || ebo == 0 {
            panic!("vao, vbo, or ebo couldn't be initialized");
        }

        unsafe {
            gl::BindVertexArray(vao);

            // bind verts array
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (verts.len() * std::mem::size_of::<GLVert>())
                    .try_into()
                    .unwrap(),
                verts.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW,
            );

            // bind indices array
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (3 * triangles.len() * std::mem::size_of::<gl::types::GLuint>())
                    .try_into()
                    .unwrap(),
                triangles.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW,
            );

            // positions at attribute location 0
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<GLVert>().try_into().unwrap(),
                offset_of!(GLVert, pos) as *const gl::types::GLvoid,
            );
            // uvs at attribute location 2
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<GLVert>().try_into().unwrap(),
                offset_of!(GLVert, uv) as *const gl::types::GLvoid,
            );
            // normals at attribute location 1
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<GLVert>().try_into().unwrap(),
                offset_of!(GLVert, normal) as *const gl::types::GLvoid,
            );

            gl::BindVertexArray(0);
        }

        Self {
            num_triangles: triangles.len(),
            vao,
        }
    }
}

impl Drawable for GLMesh {
    type ExtraData = ();
    type Error = ();

    fn draw(&self, _extra_data: &mut Self::ExtraData) -> Result<(), Self::Error> {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                (3 * self.num_triangles).try_into().unwrap(),
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
            gl::BindVertexArray(0);
        }
        Ok(())
    }

    fn draw_wireframe(&self, _extra_data: &mut Self::ExtraData) -> Result<(), Self::Error> {
        unreachable!("no wireframe support for GLMesh")
    }
}
