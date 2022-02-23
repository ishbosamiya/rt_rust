pub mod builtins;

use itertools::Itertools;
use quick_renderer::{
    bvh::{BVHDrawData, BVHTree, RayHitData, RayHitOptionalData},
    drawable::Drawable,
    gl_mesh::{self, GLMesh, GLVert},
    gpu_immediate::{GPUImmediate, GPUPrimType, GPUVertCompType, GPUVertFetchMode},
    shader,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use std::{cell::RefCell, convert::TryInto, fmt::Display, path::Path, rc::Rc, sync::Mutex};

use crate::{
    glm,
    meshio::{self, MeshIO},
    path_trace::{
        intersectable::{IntersectInfo, Intersectable},
        ray::Ray,
    },
    rasterize::Rasterize,
    util::{self, normal_apply_model_matrix, vec3_apply_model_matrix},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn apply_model_matrix(&mut self, model: &glm::DMat4) {
        self.pos = vec3_apply_model_matrix(&self.pos, model);
        // self.uv doesn't need model matrix applied
        if let Some(normal) = &self.normal {
            self.normal = Some(normal_apply_model_matrix(normal, model));
        }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Mesh {
    vertices: Vec<Vertex>,
    faces: Vec<Vec<usize>>,

    /// BVH that stores face indices
    bvh: Option<BVHTree<usize>>,
    /// OpenGL mesh for rendering, is cached upon first draw, if mesh
    /// structure changes, it should be made None
    #[serde(skip)]
    gl_mesh: Mutex<Option<GLMesh>>,
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
            gl_mesh: Mutex::new(None),
        })
    }

    pub fn read_from_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let meshio = MeshIO::read(path.as_ref())?;

        Mesh::read(&meshio)
    }

    fn draw_directional_light_shader(&self, mesh_color: glm::DVec3) -> Result<(), MeshDrawError> {
        if self.faces.is_empty() {
            return Ok(());
        }

        let directional_light_shader = shader::builtins::get_directional_light_shader()
            .as_ref()
            .unwrap();
        directional_light_shader.use_shader();

        directional_light_shader.set_vec3("material.color\0", &glm::convert(mesh_color));

        {
            let gl_mesh: &mut Option<GLMesh> = &mut self.gl_mesh.lock().unwrap();
            if gl_mesh.is_none() {
                *gl_mesh = Some(GLMesh::new(
                    &self
                        .get_vertices()
                        .iter()
                        .map(|vert| {
                            GLVert::new(
                                glm::convert(*vert.get_pos()),
                                glm::convert(vert.get_uv().unwrap()),
                                glm::convert(vert.get_normal().unwrap()),
                            )
                        })
                        .collect_vec(),
                    &self
                        .get_faces()
                        .iter()
                        .flat_map(|face| {
                            // TODO: need to find a better way to triangulate the
                            // face

                            // It doesn't make sense for a face to have only 2 verts
                            assert!(face.len() > 2);

                            let v1_index = face[0];
                            face.iter().skip(1).tuple_windows().map(
                                move |(&v2_index, &v3_index)| {
                                    gl_mesh::Triangle::new(
                                        v1_index.try_into().unwrap(),
                                        v2_index.try_into().unwrap(),
                                        v3_index.try_into().unwrap(),
                                    )
                                },
                            )
                        })
                        .collect_vec(),
                ));
            }

            gl_mesh
                .as_ref()
                .unwrap()
                .draw(&())
                .map_err(|_| MeshDrawError::ErrorWhileDrawing)?;
        }

        Ok(())
    }

    pub fn draw_mesh_vertex_normals(&self, imm: &mut GPUImmediate, length: f64, color: glm::DVec4) {
        let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
            .as_ref()
            .unwrap();
        smooth_color_3d_shader.use_shader();
        smooth_color_3d_shader.set_mat4("model\0", &glm::identity());

        let color: glm::Vec4 = glm::convert(color);

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
            self.get_vertices().len() * 2,
            smooth_color_3d_shader,
        );

        self.get_vertices().iter().for_each(|vert| {
            let pos = vert.get_pos();
            let normal = vert.get_normal().as_ref().unwrap();

            let p1 = *pos;
            let p2 = pos + length * normal;

            let p1: glm::Vec3 = glm::convert(p1);
            let p2: glm::Vec3 = glm::convert(p2);

            imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
            imm.vertex_3f(pos_attr, p1[0], p1[1], p1[2]);
            imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
            imm.vertex_3f(pos_attr, p2[0], p2[1], p2[2]);
        });

        imm.end();
    }

    /// Build BVH of the mesh
    pub fn build_bvh(&mut self, epsilon: f64) {
        let mut bvh = BVHTree::new(self.faces.len(), epsilon, 4, 8);

        self.faces.iter().enumerate().for_each(|(f_index, face)| {
            let co: Vec<_> = face
                .iter()
                .map(|v_index| *self.vertices[*v_index].get_pos())
                .collect();

            bvh.insert(f_index, &co);
        });

        bvh.balance();

        self.bvh = Some(bvh);
    }

    /// Build BVH if needed. Assumes that BVH is valid if it was built
    /// already.
    ///
    /// To force build BVH, call [`Mesh::delete_bvh()`] before this
    /// call, or call [`Mesh::build_bvh()`] directly.
    pub fn rebuild_bvh_if_needed(&mut self, epsilon: f64) {
        if self.bvh.is_none() {
            self.build_bvh(epsilon);
        }
    }

    pub fn delete_bvh(&mut self) {
        self.bvh = None;
    }

    pub fn get_bvh(&self) -> &Option<BVHTree<usize>> {
        &self.bvh
    }

    pub fn apply_model_matrix(&mut self, model: &glm::DMat4) {
        self.vertices.par_iter_mut().for_each(|vert| {
            vert.apply_model_matrix(model);
        });
    }

    pub fn get_min_max_bounds(&self) -> (glm::DVec3, glm::DVec3) {
        self.vertices.iter().fold(
            (*self.vertices[0].get_pos(), *self.vertices[0].get_pos()),
            |acc, elem| {
                (
                    glm::vec3(
                        acc.0[0].min(elem.get_pos()[0]),
                        acc.0[1].min(elem.get_pos()[1]),
                        acc.0[2].min(elem.get_pos()[2]),
                    ),
                    glm::vec3(
                        acc.1[0].max(elem.get_pos()[0]),
                        acc.1[1].max(elem.get_pos()[1]),
                        acc.1[2].max(elem.get_pos()[2]),
                    ),
                )
            },
        )
    }

    /// Get a reference to the mesh's vertices.
    pub fn get_vertices(&self) -> &[Vertex] {
        self.vertices.as_slice()
    }

    /// Get a reference to the mesh's faces.
    pub fn get_faces(&self) -> &[Vec<usize>] {
        self.faces.as_slice()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MeshDrawError {
    ErrorWhileDrawing,
    NoColorButSmoothColorShader,
    // Current decision is to not store the actual error since no
    // specific error is anyway propagated.
    ErrorWhileDrawingMeshBVH,
}

impl std::error::Error for MeshDrawError {}

impl std::fmt::Display for MeshDrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshDrawError::ErrorWhileDrawing => {
                write!(f, "Error while drawing Mesh")
            }
            MeshDrawError::NoColorButSmoothColorShader => write!(
                f,
                "No color provided in draw data but asking to use smooth color 3D shader"
            ),
            MeshDrawError::ErrorWhileDrawingMeshBVH => write!(f, "Error while drawing mesh BVH"),
        }
    }
}

impl From<()> for MeshDrawError {
    fn from(_err: ()) -> MeshDrawError {
        MeshDrawError::ErrorWhileDrawing
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum MeshUseShader {
    DirectionalLight { color: glm::DVec3 },
    SmoothColor3D,
    FaceOrientation,
}

impl Display for MeshUseShader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshUseShader::DirectionalLight { color } => {
                write!(f, "Directional Light: color: {}", color)
            }
            MeshUseShader::SmoothColor3D => write!(f, "Smooth Color 3D"),
            MeshUseShader::FaceOrientation => write!(f, "Face Orientation"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MeshBVHDrawData {
    draw_bvh: bool,
    bvh_draw_level: usize,
    bvh_color: glm::DVec4,
}

impl MeshBVHDrawData {
    pub fn new(draw_bvh: bool, bvh_draw_level: usize, bvh_color: glm::DVec4) -> Self {
        Self {
            draw_bvh,
            bvh_draw_level,
            bvh_color,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeshDrawData {
    imm: Rc<RefCell<GPUImmediate>>,
    use_shader: MeshUseShader,
    bvh_draw_data: Option<MeshBVHDrawData>,
}

impl MeshDrawData {
    pub fn new(
        imm: Rc<RefCell<GPUImmediate>>,
        use_shader: MeshUseShader,
        bvh_draw_data: Option<MeshBVHDrawData>,
    ) -> Self {
        MeshDrawData {
            imm,
            use_shader,
            bvh_draw_data,
        }
    }
}

impl Drawable for Mesh {
    type ExtraData = MeshDrawData;
    type Error = MeshDrawError;

    fn draw(&self, draw_data: &MeshDrawData) -> Result<(), MeshDrawError> {
        match draw_data.use_shader {
            MeshUseShader::DirectionalLight { color } => {
                self.draw_directional_light_shader(color)?
            }
            _ => todo!(),
        }

        if let Some(bvh_draw_data) = draw_data.bvh_draw_data {
            if bvh_draw_data.draw_bvh {
                if let Some(bvh) = &self.bvh {
                    bvh.draw(&BVHDrawData::new(
                        draw_data.imm.clone(),
                        bvh_draw_data.bvh_draw_level,
                        bvh_draw_data.bvh_color,
                    ))
                    .map_err(|_| MeshDrawError::ErrorWhileDrawingMeshBVH)?
                }
            }
        }

        Ok(())
    }

    fn draw_wireframe(&self, _draw_data: &MeshDrawData) -> Result<(), MeshDrawError> {
        todo!()
    }
}

impl Rasterize for Mesh {
    fn cleanup_opengl(&mut self) {
        if let Some(gl_mesh) = self.gl_mesh.lock().unwrap().as_mut() {
            gl_mesh.cleanup_opengl();
        }
    }
}

impl Intersectable for Mesh {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<IntersectInfo> {
        #[cfg(feature = "mesh_no_bvh")]
        {
            let mut best_hit = None;
            let mut best_hit_dist = t_max;
            self.faces
                .iter()
                .enumerate()
                .for_each(|(face_index, face)| {
                    let v1_index = face[0];
                    let v1 = &self.vertices[v1_index];
                    for (v2_index, v3_index) in face.iter().skip(1).tuple_windows() {
                        let v2 = &self.vertices[*v2_index];
                        let v3 = &self.vertices[*v3_index];

                        if let Some((dist, bary_coords)) = ray.intersect_triangle(
                            v1.get_pos(),
                            v2.get_pos(),
                            v3.get_pos(),
                            f64::EPSILON,
                        ) {
                            if dist > t_min && dist < best_hit_dist {
                                let n1 = v1.get_normal().as_ref().unwrap();
                                let n2 = v2.get_normal().as_ref().unwrap();
                                let n3 = v3.get_normal().as_ref().unwrap();
                                let mut hit_data = RayHitData::new(dist);
                                hit_data.normal =
                                    Some(vec3_apply_bary_coord(n1, n2, n3, &bary_coords));
                                hit_data
                                    .set_data(RayHitOptionalData::new(face_index, ray.at(dist)));
                                best_hit = Some(hit_data);
                                best_hit_dist = dist;
                            }
                        }
                    }
                });
            best_hit.map(|hit_data: RayHitData<usize, ()>| {
                let mut intersect_info =
                    IntersectInfo::new(hit_data.dist, hit_data.data.unwrap().co);
                intersect_info.set_normal(ray, &hit_data.normal.unwrap());
                intersect_info
            })
        }

        #[cfg(not(feature = "mesh_no_bvh"))]
        {
            #[derive(Debug, Clone, Copy)]
            struct MeshRayCastData {
                uv: glm::DVec2,
                bary_coords: glm::DVec3,
            }

            let mesh_ray_cast_callback =
                |(co, dir): (&glm::DVec3, &glm::DVec3), face_index: usize| {
                    debug_assert_eq!(ray.get_origin(), co);
                    debug_assert_eq!(ray.get_direction(), dir);
                    let face = &self.faces[face_index];
                    let v1_index = face[0];
                    let v1 = &self.vertices[v1_index];
                    for (v2_index, v3_index) in face.iter().skip(1).tuple_windows() {
                        let v2 = &self.vertices[*v2_index];
                        let v3 = &self.vertices[*v3_index];

                        if let Some((dist, bary_coords)) = ray.intersect_triangle(
                            v1.get_pos(),
                            v2.get_pos(),
                            v3.get_pos(),
                            f64::EPSILON,
                        ) {
                            if dist > t_min && dist < t_max {
                                let uv1 = v1.get_uv().as_ref().unwrap();
                                let uv2 = v2.get_uv().as_ref().unwrap();
                                let uv3 = v3.get_uv().as_ref().unwrap();
                                let n1 = v1.get_normal().as_ref().unwrap();
                                let n2 = v2.get_normal().as_ref().unwrap();
                                let n3 = v3.get_normal().as_ref().unwrap();
                                let mut hit_data = RayHitData::new(dist);
                                hit_data.normal =
                                    Some(util::vec3_apply_bary_coord(n1, n2, n3, &bary_coords));
                                hit_data
                                    .set_data(RayHitOptionalData::new(face_index, ray.at(dist)));
                                hit_data.set_extra_data(MeshRayCastData {
                                    uv: util::vec2_apply_bary_coord(uv1, uv2, uv3, &bary_coords),
                                    bary_coords,
                                });
                                return Some(hit_data);
                            }
                        }
                    }
                    None
                };

            self.get_bvh()
                .as_ref()
                .unwrap()
                .ray_cast(
                    *ray.get_origin(),
                    *ray.get_direction(),
                    Some(&mesh_ray_cast_callback),
                )
                .map(|hit_data: RayHitData<usize, MeshRayCastData>| {
                    let mut intersect_info = IntersectInfo::new(
                        hit_data.dist,
                        hit_data.data.unwrap().co,
                        hit_data.extra_data.unwrap().bary_coords,
                    );
                    intersect_info.set_uv(hit_data.extra_data.unwrap().uv);
                    intersect_info.set_normal(ray, &hit_data.normal.unwrap());
                    intersect_info
                })
        }
    }
}
