pub mod builtins;

use std::cell::RefCell;
use std::path::Path;
use std::{fmt::Display, rc::Rc};

use itertools::Itertools;

use crate::{
    bvh::{BVHDrawData, BVHTree},
    glm,
    meshio::{self, MeshIO},
    rasterize::{
        drawable::Drawable,
        gpu_immediate::{GPUImmediate, GPUPrimType, GPUVertCompType, GPUVertFetchMode},
        shader,
    },
};

pub struct Vertex {
    pos: glm::DVec3,
    uv: Option<glm::DVec2>,
    normal: Option<glm::DVec3>,
}

impl Vertex {
    pub fn new(pos: glm::DVec3) -> Self {
        Self {
            pos,
            uv: None,
            normal: None,
        }
    }

    pub fn new_maybe_all(
        pos: glm::DVec3,
        uv: Option<glm::DVec2>,
        normal: Option<glm::DVec3>,
    ) -> Self {
        Self { pos, uv, normal }
    }

    pub fn new_with_uv(pos: glm::DVec3, uv: glm::DVec2) -> Self {
        Self {
            pos,
            uv: Some(uv),
            normal: None,
        }
    }

    pub fn new_with_normal(pos: glm::DVec3, normal: glm::DVec3) -> Self {
        Self {
            pos,
            uv: None,
            normal: Some(normal),
        }
    }

    pub fn new_with_uv_and_normal(pos: glm::DVec3, uv: glm::DVec2, normal: glm::DVec3) -> Self {
        Self {
            pos,
            uv: Some(uv),
            normal: Some(normal),
        }
    }

    pub fn set_pos(&mut self, pos: glm::DVec3) {
        self.pos = pos;
    }

    pub fn set_uv(&mut self, uv: glm::DVec2) {
        self.uv = Some(uv);
    }

    pub fn set_normal(&mut self, normal: glm::DVec3) {
        self.normal = Some(normal);
    }

    pub fn get_pos(&self) -> &glm::DVec3 {
        &self.pos
    }

    pub fn get_uv(&self) -> &Option<glm::DVec2> {
        &self.uv
    }

    pub fn get_normal(&self) -> &Option<glm::DVec3> {
        &self.normal
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    MeshIO(meshio::MeshIOError),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MeshIO(error) => write!(f, "{}", error),
        }
    }
}

impl From<meshio::MeshIOError> for Error {
    fn from(error: meshio::MeshIOError) -> Self {
        Self::MeshIO(error)
    }
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    faces: Vec<Vec<usize>>,

    // BVH that stores face indices
    bvh: Option<BVHTree<usize>>,
}

impl Mesh {
    pub fn read(meshio: &MeshIO) -> Result<Self, Error> {
        let vertices = meshio
            .face_indices
            .iter()
            .flat_map(|face| {
                face.iter().map(|(pos_index, uv_index, normal_index)| {
                    let pos = meshio.positions[*pos_index];
                    let uv;
                    if *uv_index != usize::MAX {
                        uv = Some(meshio.uvs[*uv_index]);
                    } else {
                        uv = None;
                    }
                    let normal;
                    if *normal_index != usize::MAX {
                        normal = Some(meshio.normals[*normal_index]);
                    } else {
                        normal = None;
                    }

                    Vertex::new_maybe_all(pos, uv, normal)
                })
            })
            .collect();

        let mut vertex_counter = 0;
        let faces = meshio
            .face_indices
            .iter()
            .map(|face| {
                face.iter()
                    .map(|_| {
                        let vertex_index = vertex_counter;
                        vertex_counter += 1;
                        vertex_index
                    })
                    .collect()
            })
            .collect();

        Ok(Self {
            vertices,
            faces,
            bvh: None,
        })
    }

    pub fn read_from_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let meshio = MeshIO::read(path.as_ref())?;

        Mesh::read(&meshio)
    }

    fn draw_directional_light_shader(
        &self,
        draw_data: &mut MeshDrawData,
    ) -> Result<(), MeshDrawError> {
        if self.faces.is_empty() {
            return Ok(());
        }

        let directional_light_shader = shader::builtins::get_directional_light_shader()
            .as_ref()
            .unwrap();
        directional_light_shader.use_shader();

        let mut imm = draw_data.imm.borrow_mut();

        let format = imm.get_cleared_vertex_format();
        let pos_attr = format.add_attribute(
            "in_pos\0".to_string(),
            GPUVertCompType::F32,
            3,
            GPUVertFetchMode::Float,
        );
        // let uv_attr = format.add_attribute(
        //     "in_uv\0".to_string(),
        //     GPUVertCompType::F32,
        //     2,
        //     GPUVertFetchMode::Float,
        // );
        let normal_attr = format.add_attribute(
            "in_normal\0".to_string(),
            GPUVertCompType::F32,
            3,
            GPUVertFetchMode::Float,
        );

        // currently assuming that no face has verts in excess of 10
        imm.begin_at_most(
            GPUPrimType::Tris,
            self.faces.len() * 10,
            directional_light_shader,
        );

        self.faces.iter().for_each(|face| {
            // currently assuming that no face has verts in excess of
            // 10, will figure out a generic way to handle this later
            assert!(face.len() <= 10);

            // It doesn't make sense for a face to have only 2 verts
            assert!(face.len() > 2);

            let v1_index = face[0];
            let v1 = &self.vertices[v1_index];
            for (v2_index, v3_index) in face.iter().skip(1).tuple_windows() {
                let v2 = &self.vertices[*v2_index];
                let v3 = &self.vertices[*v3_index];

                let v1_normal: glm::Vec3 = glm::convert(v1.normal.unwrap());
                imm.attr_3f(normal_attr, v1_normal[0], v1_normal[1], v1_normal[2]);
                // let v1_uv: glm::Vec2 = glm::convert(v1.uv.unwrap());
                // imm.attr_2f(uv_attr, v1_uv[0], v1_uv[1]);
                let v1_pos: glm::Vec3 = glm::convert(v1.pos);
                imm.vertex_3f(pos_attr, v1_pos[0], v1_pos[1], v1_pos[2]);

                let v2_normal: glm::Vec3 = glm::convert(v2.normal.unwrap());
                imm.attr_3f(normal_attr, v2_normal[0], v2_normal[1], v2_normal[2]);
                // let v2_uv: glm::Vec2 = glm::convert(v2.uv.unwrap());
                // imm.attr_2f(uv_attr, v2_uv[0], v2_uv[1]);
                let v2_pos: glm::Vec3 = glm::convert(v2.pos);
                imm.vertex_3f(pos_attr, v2_pos[0], v2_pos[1], v2_pos[2]);

                let v3_normal: glm::Vec3 = glm::convert(v3.normal.unwrap());
                imm.attr_3f(normal_attr, v3_normal[0], v3_normal[1], v3_normal[2]);
                // let v3_uv: glm::Vec2 = glm::convert(v3.uv.unwrap());
                // imm.attr_2f(uv_attr, v3_uv[0], v3_uv[1]);
                let v3_pos: glm::Vec3 = glm::convert(v3.pos);
                imm.vertex_3f(pos_attr, v3_pos[0], v3_pos[1], v3_pos[2]);
            }
        });

        imm.end();

        Ok(())
    }

    pub fn build_bvh(&mut self, epsilon: f64) {
        let mut bvh = BVHTree::new(self.faces.len(), epsilon, 4, 8);

        self.faces.iter().enumerate().for_each(|(f_index, face)| {
            let co = face
                .iter()
                .map(|v_index| *self.vertices[*v_index].get_pos())
                .collect();

            bvh.insert(f_index, co);
        });

        bvh.balance();

        self.bvh = Some(bvh);
    }

    pub fn get_bvh(&self) -> &Option<BVHTree<usize>> {
        &self.bvh
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MeshDrawError {
    GenerateGLMeshFirst,
    ErrorWhileDrawing,
    NoColorButSmoothColorShader,
}

impl std::error::Error for MeshDrawError {}

impl std::fmt::Display for MeshDrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshDrawError::GenerateGLMeshFirst => {
                write!(f, "Generate GLMesh before calling draw()")
            }
            MeshDrawError::ErrorWhileDrawing => {
                write!(f, "Error while drawing Mesh")
            }
            MeshDrawError::NoColorButSmoothColorShader => write!(
                f,
                "No color provided in draw data but asking to use smooth color 3D shader"
            ),
        }
    }
}

impl From<()> for MeshDrawError {
    fn from(_err: ()) -> MeshDrawError {
        MeshDrawError::ErrorWhileDrawing
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MeshUseShader {
    DirectionalLight,
    SmoothColor3D,
    FaceOrientation,
}

impl Display for MeshUseShader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshUseShader::DirectionalLight => write!(f, "Directional Light"),
            MeshUseShader::SmoothColor3D => write!(f, "Smooth Color 3D"),
            MeshUseShader::FaceOrientation => write!(f, "Face Orientation"),
        }
    }
}

pub struct MeshDrawData {
    imm: Rc<RefCell<GPUImmediate>>,
    use_shader: MeshUseShader,
    draw_bvh: bool,
    bvh_draw_level: usize,
    bvh_color: glm::DVec4,
}

impl MeshDrawData {
    pub fn new(
        imm: Rc<RefCell<GPUImmediate>>,
        use_shader: MeshUseShader,
        draw_bvh: bool,
        bvh_draw_level: usize,
        bvh_color: glm::DVec4,
    ) -> Self {
        MeshDrawData {
            imm,
            use_shader,
            draw_bvh,
            bvh_draw_level,
            bvh_color,
        }
    }
}

impl Drawable for Mesh {
    type ExtraData = MeshDrawData;
    type Error = MeshDrawError;

    fn draw(&self, draw_data: &mut MeshDrawData) -> Result<(), MeshDrawError> {
        match draw_data.use_shader {
            MeshUseShader::DirectionalLight => self.draw_directional_light_shader(draw_data)?,
            // MeshUseShader::SmoothColor3D => self.draw_smooth_color_3d_shader(draw_data),
            // MeshUseShader::FaceOrientation => self.draw_face_orientation_shader(draw_data),
            _ => todo!(),
        }

        if draw_data.draw_bvh {
            if let Some(bvh) = &self.bvh {
                bvh.draw(&mut BVHDrawData::new(
                    draw_data.imm.clone(),
                    draw_data.bvh_draw_level,
                    draw_data.bvh_color,
                ))?
            }
        }

        Ok(())
    }

    fn draw_wireframe(&self, _draw_data: &mut MeshDrawData) -> Result<(), MeshDrawError> {
        todo!()
    }
}
