pub mod builtins;

use std::fmt::Display;
use std::path::Path;

use itertools::Itertools;

use crate::{
    bvh::{BVHDrawData, BVHTree},
    drawable::Drawable,
    glm,
    gpu_immediate::{GPUImmediate, GPUPrimType, GPUVertCompType, GPUVertFetchMode},
    meshio::{self, MeshIO},
    shader,
};

pub struct Vertex {
    pos: glm::DVec3,
    uv: Option<glm::DVec2>,
    normal: Option<glm::DVec3>,
    tangent: Option<glm::DVec3>,
    bitangent: Option<glm::DVec3>,
}

impl Vertex {
    pub fn new(pos: glm::DVec3) -> Self {
        Self {
            pos,
            uv: None,
            normal: None,
            tangent: None,
            bitangent: None,
        }
    }

    pub fn new_maybe_all(
        pos: glm::DVec3,
        uv: Option<glm::DVec2>,
        normal: Option<glm::DVec3>,
        tangent: Option<glm::DVec3>,
        bitangent: Option<glm::DVec3>,
    ) -> Self {
        Self {
            pos,
            uv,
            normal,
            tangent,
            bitangent,
        }
    }

    pub fn new_with_uv(pos: glm::DVec3, uv: glm::DVec2) -> Self {
        Self {
            pos,
            uv: Some(uv),
            normal: None,
            tangent: None,
            bitangent: None,
        }
    }

    pub fn new_with_normal(pos: glm::DVec3, normal: glm::DVec3) -> Self {
        Self {
            pos,
            uv: None,
            normal: Some(normal),
            tangent: None,
            bitangent: None,
        }
    }

    pub fn new_with_uv_and_normal(pos: glm::DVec3, uv: glm::DVec2, normal: glm::DVec3) -> Self {
        Self {
            pos,
            uv: Some(uv),
            normal: Some(normal),
            tangent: None,
            bitangent: None,
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

    pub fn set_tangent(&mut self, tangent: glm::DVec3) {
        self.tangent = Some(tangent);
    }

    pub fn set_bitangent(&mut self, bitangent: glm::DVec3) {
        self.bitangent = Some(bitangent);
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

    pub fn get_tangent(&self) -> &Option<glm::DVec3> {
        &self.tangent
    }

    pub fn get_bitangent(&self) -> &Option<glm::DVec3> {
        &self.bitangent
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

                    Vertex::new_maybe_all(pos, uv, normal, None, None)
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

    pub fn get_verticies(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn get_faces(&self) -> &Vec<Vec<usize>> {
        &self.faces
    }

    fn draw_directional_light_shader(
        &self,
        draw_data: &mut MeshDrawData,
    ) -> Result<(), MeshDrawError> {
        if self.faces.is_empty() {
            return Ok(());
        }

        let imm = &mut draw_data.imm;
        let directional_light_shader = shader::builtins::get_directional_light_shader()
            .as_ref()
            .unwrap();

        directional_light_shader.use_shader();

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

    pub fn calculate_tangent_info(&mut self) {
        // TODO(ish): if uv or normal info is not available at the
        // vertex, need to return Err

        for face in &self.faces {
            // TODO(ish): make it work properly for concave faces

            // It doesn't make sense for a face to have only 2 verts
            assert!(face.len() > 2);

            let v1_index = face[0];
            for (v2_index, v3_index) in face.iter().skip(1).tuple_windows() {
                let v1 = &self.vertices[v1_index];
                let v2 = &self.vertices[*v2_index];
                let v3 = &self.vertices[*v3_index];

                let pos1 = v1.get_pos();
                let pos2 = v2.get_pos();
                let pos3 = v3.get_pos();

                let uv1 = v1.get_uv().as_ref().unwrap();
                let uv2 = v2.get_uv().as_ref().unwrap();
                let uv3 = v3.get_uv().as_ref().unwrap();

                let edge1 = pos2 - pos1;
                let edge2 = pos3 - pos1;
                let deltauv1 = uv2 - uv1;
                let deltauv2 = uv3 - uv1;

                let f = 1.0 / (deltauv1[0] * deltauv2[1] - deltauv2[0] * deltauv1[1]);

                let tangent = f * glm::vec3(
                    deltauv2[1] * edge1[0] - deltauv1[1] * edge2[0],
                    deltauv2[1] * edge1[1] - deltauv1[1] * edge2[1],
                    deltauv2[1] * edge1[2] - deltauv1[1] * edge2[2],
                );

                let bitangent = f * glm::vec3(
                    -deltauv2[0] * edge1[0] + deltauv1[0] * edge2[0],
                    -deltauv2[0] * edge1[1] + deltauv1[0] * edge2[1],
                    -deltauv2[0] * edge1[2] + deltauv1[0] * edge2[2],
                );

                let v1 = &mut self.vertices[v1_index];
                v1.set_tangent(tangent);
                v1.set_bitangent(bitangent);

                let v2 = &mut self.vertices[*v2_index];
                v2.set_tangent(tangent);
                v2.set_bitangent(bitangent);

                let v3 = &mut self.vertices[*v3_index];
                v3.set_tangent(tangent);
                v3.set_bitangent(bitangent);
            }
        }
    }

    pub fn draw_normals(&self, imm: &mut GPUImmediate, color: glm::DVec4, scale_factor: f64) {
        let color: glm::Vec4 = glm::convert(color);
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
            self.get_verticies().len() * 2,
            smooth_color_3d_shader,
        );

        self.get_verticies().iter().for_each(|vert| {
            let pos: glm::Vec3 = glm::convert(*vert.get_pos());
            let normal: glm::Vec3 = glm::convert(vert.get_normal().unwrap());
            let end_pos = pos + scale_factor as f32 * normal.normalize();

            imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
            imm.vertex_3f(pos_attr, pos[0], pos[1], pos[2]);

            imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
            imm.vertex_3f(pos_attr, end_pos[0], end_pos[1], end_pos[2]);
        });

        imm.end();
    }

    pub fn draw_tangents(&self, imm: &mut GPUImmediate, color: glm::DVec4, scale_factor: f64) {
        let color: glm::Vec4 = glm::convert(color);
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
            self.get_verticies().len() * 2,
            smooth_color_3d_shader,
        );

        self.get_verticies().iter().for_each(|vert| {
            let pos: glm::Vec3 = glm::convert(*vert.get_pos());
            let tangent: glm::Vec3 = glm::convert(vert.get_tangent().unwrap());
            let end_pos = pos + scale_factor as f32 * tangent.normalize();

            imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
            imm.vertex_3f(pos_attr, pos[0], pos[1], pos[2]);

            imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
            imm.vertex_3f(pos_attr, end_pos[0], end_pos[1], end_pos[2]);
        });

        imm.end();
    }

    pub fn draw_bitangents(&self, imm: &mut GPUImmediate, color: glm::DVec4, scale_factor: f64) {
        let color: glm::Vec4 = glm::convert(color);
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
            self.get_verticies().len() * 2,
            smooth_color_3d_shader,
        );

        self.get_verticies().iter().for_each(|vert| {
            let pos: glm::Vec3 = glm::convert(*vert.get_pos());
            let bitangent: glm::Vec3 = glm::convert(vert.get_bitangent().unwrap());
            let end_pos = pos + scale_factor as f32 * bitangent.normalize();

            imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
            imm.vertex_3f(pos_attr, pos[0], pos[1], pos[2]);

            imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
            imm.vertex_3f(pos_attr, end_pos[0], end_pos[1], end_pos[2]);
        });

        imm.end();
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

pub struct MeshDrawData<'a> {
    imm: &'a mut GPUImmediate,
    use_shader: MeshUseShader,
    draw_bvh: bool,
    bvh_draw_level: usize,
    bvh_color: glm::DVec4,
    _color: Option<glm::Vec4>,
}

impl<'a> MeshDrawData<'a> {
    pub fn new(
        imm: &'a mut GPUImmediate,
        use_shader: MeshUseShader,
        draw_bvh: bool,
        bvh_draw_level: usize,
        bvh_color: glm::DVec4,
        color: Option<glm::Vec4>,
    ) -> Self {
        MeshDrawData {
            imm,
            use_shader,
            draw_bvh,
            bvh_draw_level,
            bvh_color,
            _color: color,
        }
    }
}

impl Drawable<MeshDrawData<'_>, MeshDrawError> for Mesh {
    fn draw(&self, draw_data: &mut MeshDrawData<'_>) -> Result<(), MeshDrawError> {
        match draw_data.use_shader {
            MeshUseShader::DirectionalLight => self.draw_directional_light_shader(draw_data)?,
            // MeshUseShader::SmoothColor3D => self.draw_smooth_color_3d_shader(draw_data),
            // MeshUseShader::FaceOrientation => self.draw_face_orientation_shader(draw_data),
            _ => todo!(),
        }

        if draw_data.draw_bvh {
            if let Some(bvh) = &self.bvh {
                bvh.draw(&mut BVHDrawData::new(
                    draw_data.imm,
                    draw_data.bvh_draw_level,
                    draw_data.bvh_color,
                ))?
            }
        }

        Ok(())
    }

    fn draw_wireframe(&self, _draw_data: &mut MeshDrawData<'_>) -> Result<(), MeshDrawError> {
        todo!()
    }
}
