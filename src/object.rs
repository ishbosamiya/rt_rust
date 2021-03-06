use lazy_static::lazy_static;
use quick_renderer::{
    drawable::{Drawable, NoSpecificDrawError},
    gpu_immediate::GPUImmediate,
    rasterize::Rasterize,
};
use serde::{Deserialize, Serialize};

use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::Rc,
    sync::Mutex,
};

#[cfg(feature = "use_embree")]
use crate::embree::Embree;
use crate::{
    glm,
    mesh::MeshDrawError,
    namegen::NameGen,
    path_trace::{intersectable::Intersectable, shader_list::ShaderID as PathTraceShaderID},
};

lazy_static! {
    static ref SPHERE_NAME_GEN: Mutex<NameGen> = Mutex::new(NameGen::new("sphere".to_string()));
    static ref MESH_NAME_GEN: Mutex<NameGen> = Mutex::new(NameGen::new("mesh".to_string()));
}

/// A unique identifier given to each [`Object`] during its
/// initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjectID(usize);

impl ObjectID {
    /// # Safety
    ///
    /// The actual usize stored can be random, so doesn't make sense
    /// to create it from raw most of the time. Only Scene must create
    /// an ObjectID from raw.
    pub unsafe fn from_raw(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
pub enum DrawError {
    Mesh(MeshDrawError),
    Sphere(NoSpecificDrawError),
}

impl Display for DrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrawError::Mesh(err) => write!(f, "DrawError: Mesh: {}", err),
            DrawError::Sphere(err) => write!(f, "DrawError: Sphere: {}", err),
        }
    }
}

impl std::error::Error for DrawError {}

#[derive(Debug, Clone)]
pub struct ObjectDrawData {
    imm: Rc<RefCell<GPUImmediate>>,

    /// Must set the model matrix in the shader if `use_model_matrix`
    /// is true, otherwise set the model matrix to be
    /// `glm::identity()`
    ///
    /// It is possible for the Scene to have applied the model matrix
    /// to the object and thus using the model matrix again in the
    /// OpenGL shaders can lead to problems.
    ///
    /// TODO: need to find a better way to handle this.
    use_model_matrix: bool,

    /// Color of object
    viewport_color: glm::DVec4,
}

impl ObjectDrawData {
    pub fn new(imm: Rc<RefCell<GPUImmediate>>, viewport_color: glm::DVec4) -> Self {
        Self {
            imm,
            use_model_matrix: true,
            viewport_color,
        }
    }

    /// Sets `use_model_matrix` of `Self`
    ///
    /// # Safety
    /// See comments on `Self::use_model_matrix` for more
    /// details on why this needs to exist.
    pub unsafe fn set_use_model_matrix(&mut self, use_model_matrix: bool) {
        self.use_model_matrix = use_model_matrix;
    }

    pub fn set_viewport_color(&mut self, viewport_color: glm::DVec4) {
        self.viewport_color = viewport_color;
    }
}

pub enum PrimitiveType {
    Triangle,
}

/// Data required to fetch the parameters of the object that must be
/// interpolated for the point `position` on the object
pub struct DataForInterpolation {
    /// Index of the primitive for which UVs must be calculated
    primitive_index: Option<usize>,
    primitive_type: PrimitiveType,
    /// Barycentric coords of the point on the primitive
    bary_coords: glm::DVec3,

    /// Position on the object for which UVs must be calculated
    position: glm::DVec3,
}

impl DataForInterpolation {
    pub fn new(
        primitive_index: Option<usize>,
        primitive_type: PrimitiveType,
        bary_coords: glm::DVec3,
        position: glm::DVec3,
    ) -> Self {
        Self {
            primitive_index,
            primitive_type,
            bary_coords,
            position,
        }
    }
}

#[typetag::serde(tag = "type")]
pub trait Object:
    Debug
    + Intersectable
    + Drawable<ExtraData = ObjectDrawData, Error = DrawError>
    + Rasterize
    + Sync
    + Send
{
    fn set_model_matrix(&mut self, model: glm::DMat4);
    fn get_model_matrix(&self) -> &Option<glm::DMat4>;
    fn apply_model_matrix(&mut self);
    fn unapply_model_matrix(&mut self) {
        let inv_model = glm::inverse(&self.get_model_matrix().unwrap());
        let model = self.get_model_matrix().unwrap();
        self.set_model_matrix(inv_model);
        self.apply_model_matrix();
        self.set_model_matrix(model);
    }

    fn set_path_trace_shader_id(&mut self, shader_id: PathTraceShaderID);
    fn get_path_trace_shader_id(&self) -> Option<PathTraceShaderID>;

    fn set_object_id(&mut self, object_id: ObjectID);
    fn get_object_id(&self) -> ObjectID;

    /// Get mutable reference to the name of the object
    fn get_object_name_mut(&mut self) -> &mut String;
    /// Get reference to the name of the object
    fn get_object_name(&self) -> &str;

    fn get_min_max_bounds(&self) -> (glm::DVec3, glm::DVec3);

    #[cfg(feature = "use_embree")]
    fn add_object_to_embree(&self, embree: &mut Embree);

    /// Get UV of the object with the given data
    fn get_uv(&self, data: &DataForInterpolation) -> glm::DVec2;

    /// Get normal of the object with the given data
    fn get_normal(&self, data: &DataForInterpolation) -> glm::DVec3;

    /// Must set any data that must be cached
    fn set_cached_data(&mut self);

    /// Rebuild BVH of the object if needed
    ///
    /// TODO: It might make sense to add support for storing at what
    /// state (maybe through some hash) the BVH was built. This can
    /// even evolve to creating BVH in multiple states and choosing
    /// the state that is most appropriate for that particular BVH
    /// query.
    ///
    /// TODO: add delete BVH function in the Object Trait
    fn rebuild_bvh_if_needed(&mut self, epsilon: f64);

    /// Returns the trait object to be used as an [`std::any::Any`]
    ///
    /// This is specially useful when the object is used through
    /// dynamic dispatch and need to work on the object itself instead
    /// of the trait object. Done via `std::any::Any::downcast_ref()`.
    ///
    /// See
    /// <https://stackoverflow.com/questions/33687447/how-to-get-a-reference-to-a-concrete-type-from-a-trait-object>
    /// for more details on why this specific method is needed in the
    /// trait.
    ///
    /// Implementation for this function should always be
    /// ```text
    /// fn as_any(&self) -> &dyn std::any::Any {
    ///   self
    /// }
    /// ```
    fn as_any(&self) -> &dyn std::any::Any;
}

pub mod objects {
    pub use mesh::Mesh;
    pub use sphere::Sphere;

    mod sphere {
        use quick_renderer::{
            drawable::{Drawable, NoSpecificDrawError},
            rasterize::Rasterize,
        };
        use serde::{Deserialize, Serialize};

        #[cfg(feature = "use_embree")]
        use crate::embree::Embree;
        use crate::{
            glm,
            object::{DataForInterpolation, ObjectID},
            path_trace::{
                self,
                intersectable::{IntersectInfo, Intersectable},
                ray::Ray,
                shader_list::ShaderID,
            },
            sphere::{Sphere as SphereData, SphereDrawData},
        };

        use super::super::{DrawError, Object, ObjectDrawData, SPHERE_NAME_GEN};

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Sphere {
            data: SphereData,
            shader_id: Option<ShaderID>,
            object_id: Option<ObjectID>,
            #[serde(default = "default_object_name")]
            object_name: String,
            model_matrix: Option<glm::DMat4>,

            // TODO: since this is a partial copy of SphereDrawData, it
            // might make sense to store this in a separate structure and
            // use that
            outside_color: glm::Vec4,
            inside_color: glm::Vec4,
        }

        fn default_object_name() -> String {
            SPHERE_NAME_GEN.lock().unwrap().next().unwrap()
        }

        impl Sphere {
            pub fn new(
                data: SphereData,
                outside_color: glm::Vec4,
                inside_color: glm::Vec4,
            ) -> Self {
                Self {
                    data,
                    shader_id: None,
                    object_id: None,
                    object_name: SPHERE_NAME_GEN.lock().unwrap().next().unwrap(),
                    model_matrix: None,
                    outside_color,
                    inside_color,
                }
            }
        }

        impl Intersectable for Sphere {
            fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<IntersectInfo> {
                self.data.hit(ray, t_min, t_max).map(|mut info| {
                    info.set_shader_id(self.get_path_trace_shader_id());
                    info.set_object_id(self.get_object_id());
                    info
                })
            }
        }

        impl Drawable for Sphere {
            type ExtraData = ObjectDrawData;
            type Error = DrawError;

            fn draw(&self, extra_data: &ObjectDrawData) -> Result<(), DrawError> {
                let model = if extra_data.use_model_matrix {
                    self.get_model_matrix().unwrap()
                } else {
                    glm::identity()
                };

                self.data
                    .draw(&SphereDrawData::new(
                        extra_data.imm.clone(),
                        model,
                        glm::convert(extra_data.viewport_color),
                        self.inside_color,
                    ))
                    .map_err(|_error| DrawError::Sphere(NoSpecificDrawError))?;

                Ok(())
            }

            fn draw_wireframe(&self, extra_data: &ObjectDrawData) -> Result<(), DrawError> {
                let model = if extra_data.use_model_matrix {
                    self.get_model_matrix().unwrap()
                } else {
                    glm::identity()
                };

                self.data
                    .draw_wireframe(&SphereDrawData::new(
                        extra_data.imm.clone(),
                        model,
                        self.outside_color,
                        self.inside_color,
                    ))
                    .map_err(|_error| DrawError::Sphere(NoSpecificDrawError))
            }
        }

        impl Rasterize for Sphere {
            fn cleanup_opengl(&mut self) {
                // no clean up for Sphere
            }
        }

        #[typetag::serde]
        impl Object for Sphere {
            fn set_model_matrix(&mut self, model: glm::DMat4) {
                self.model_matrix = Some(model);
            }

            fn get_model_matrix(&self) -> &Option<glm::DMat4> {
                &self.model_matrix
            }

            fn apply_model_matrix(&mut self) {
                let model = self.get_model_matrix().unwrap();
                self.data.apply_model_matrix(&model);
            }

            fn set_path_trace_shader_id(&mut self, shader_id: ShaderID) {
                self.shader_id = Some(shader_id)
            }

            fn get_path_trace_shader_id(&self) -> Option<ShaderID> {
                self.shader_id
            }

            fn set_object_id(&mut self, object_id: ObjectID) {
                self.object_id = Some(object_id);
            }

            fn get_object_id(&self) -> ObjectID {
                self.object_id.unwrap()
            }

            fn get_object_name_mut(&mut self) -> &mut String {
                &mut self.object_name
            }

            fn get_object_name(&self) -> &str {
                &self.object_name
            }

            fn get_min_max_bounds(&self) -> (glm::DVec3, glm::DVec3) {
                (
                    self.data.get_center()
                        + glm::vec3(
                            -self.data.get_radius(),
                            -self.data.get_radius(),
                            -self.data.get_radius(),
                        ),
                    self.data.get_center()
                        + glm::vec3(
                            self.data.get_radius(),
                            self.data.get_radius(),
                            self.data.get_radius(),
                        ),
                )
            }

            #[cfg(feature = "use_embree")]
            fn add_object_to_embree(&self, embree: &mut Embree) {
                embree.add_sphere(&self.data, self.get_object_id());
            }

            fn get_uv(&self, data: &DataForInterpolation) -> glm::DVec2 {
                path_trace::direction_to_equirectangular(&(data.position - self.data.get_center()))
            }

            fn get_normal(&self, data: &DataForInterpolation) -> glm::DVec3 {
                (data.position - self.data.get_center()) / self.data.get_radius()
            }

            fn set_cached_data(&mut self) {
                // no caching for sphere
            }

            fn rebuild_bvh_if_needed(&mut self, _epsilon: f64) {
                // no BVH for sphere
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    }

    mod mesh {
        use quick_renderer::{drawable::Drawable, rasterize::Rasterize, shader};

        #[cfg(feature = "use_embree")]
        use crate::embree::Embree;
        use crate::{
            glm,
            mesh::{Mesh as MeshData, MeshBVHDrawData, MeshDrawData, MeshUseShader},
            object::{DataForInterpolation, ObjectID, PrimitiveType},
            path_trace::{
                intersectable::{IntersectInfo, Intersectable},
                ray::Ray,
                shader_list::ShaderID,
            },
            util,
        };

        use super::super::{DrawError, Object, ObjectDrawData, MESH_NAME_GEN};

        use itertools::Itertools;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone)]
        struct Triangle {
            i1: usize,
            i2: usize,
            i3: usize,
        }

        impl Triangle {
            fn new(i1: usize, i2: usize, i3: usize) -> Self {
                Self { i1, i2, i3 }
            }
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Mesh {
            data: MeshData,
            /// precomputed triangulation of the mesh, valid only
            /// until mesh structure remains the same
            #[serde(skip)]
            triangles: Option<Vec<Triangle>>,
            shader_id: Option<ShaderID>,
            object_id: Option<ObjectID>,
            #[serde(default = "default_object_name")]
            object_name: String,
            model_matrix: Option<glm::DMat4>,

            // TODO: since this is a partial copy of MeshDrawData, it
            // might make sense to store this in a separate structure and
            // use that
            use_shader: MeshUseShader,
            bvh_draw_data: Option<MeshBVHDrawData>,
        }

        fn default_object_name() -> String {
            MESH_NAME_GEN.lock().unwrap().next().unwrap()
        }

        impl Mesh {
            pub fn new(
                data: MeshData,
                use_shader: MeshUseShader,
                bvh_draw_data: Option<MeshBVHDrawData>,
            ) -> Self {
                Self {
                    data,
                    triangles: None,
                    shader_id: None,
                    object_id: None,
                    object_name: MESH_NAME_GEN.lock().unwrap().next().unwrap(),
                    model_matrix: None,

                    use_shader,
                    bvh_draw_data,
                }
            }

            /// Get a reference to the mesh's data.
            pub fn get_data(&self) -> &MeshData {
                &self.data
            }
        }

        impl Intersectable for Mesh {
            fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<IntersectInfo> {
                self.data.hit(ray, t_min, t_max).map(|mut info| {
                    info.set_shader_id(self.get_path_trace_shader_id());
                    info.set_object_id(self.get_object_id());
                    info
                })
            }
        }

        impl Drawable for Mesh {
            type ExtraData = ObjectDrawData;
            type Error = DrawError;

            fn draw(&self, extra_data: &ObjectDrawData) -> Result<(), DrawError> {
                let shader = match self.use_shader {
                    MeshUseShader::DirectionalLight { color: _ } => {
                        let shader = shader::builtins::get_directional_light_shader()
                            .as_ref()
                            .unwrap();
                        shader.use_shader();
                        shader
                    }
                    _ => todo!(),
                };

                let model = if extra_data.use_model_matrix {
                    self.get_model_matrix().unwrap()
                } else {
                    glm::identity()
                };

                shader.set_mat4("model\0", &glm::convert(model));

                self.data
                    .draw(&MeshDrawData::new(
                        extra_data.imm.clone(),
                        MeshUseShader::DirectionalLight {
                            color: glm::vec4_to_vec3(&extra_data.viewport_color),
                        },
                        self.bvh_draw_data,
                    ))
                    .map_err(DrawError::Mesh)
            }

            fn draw_wireframe(&self, _extra_data: &ObjectDrawData) -> Result<(), DrawError> {
                todo!()
            }
        }

        impl Rasterize for Mesh {
            fn cleanup_opengl(&mut self) {
                self.data.cleanup_opengl();
            }
        }

        #[typetag::serde]
        impl Object for Mesh {
            fn set_model_matrix(&mut self, model: glm::DMat4) {
                self.model_matrix = Some(model);
            }

            fn get_model_matrix(&self) -> &Option<glm::DMat4> {
                &self.model_matrix
            }

            fn apply_model_matrix(&mut self) {
                let model = self.get_model_matrix().unwrap();
                self.data.apply_model_matrix(&model);
            }

            fn set_path_trace_shader_id(&mut self, shader_id: ShaderID) {
                self.shader_id = Some(shader_id)
            }

            fn get_path_trace_shader_id(&self) -> Option<ShaderID> {
                self.shader_id
            }

            fn set_object_id(&mut self, object_id: ObjectID) {
                self.object_id = Some(object_id);
            }

            fn get_object_id(&self) -> ObjectID {
                self.object_id.unwrap()
            }

            fn get_object_name_mut(&mut self) -> &mut String {
                &mut self.object_name
            }

            fn get_object_name(&self) -> &str {
                &self.object_name
            }

            fn get_min_max_bounds(&self) -> (glm::DVec3, glm::DVec3) {
                self.data.get_min_max_bounds()
            }

            #[cfg(feature = "use_embree")]
            fn add_object_to_embree(&self, embree: &mut Embree) {
                embree.add_mesh(&self.data, self.get_object_id());
            }

            fn get_uv(&self, data: &DataForInterpolation) -> glm::DVec2 {
                match data.primitive_type {
                    PrimitiveType::Triangle => {
                        let triangle = &self.triangles.as_ref().expect("not cached yet")
                            [data.primitive_index.unwrap()];
                        let v1 = &self.data.get_vertices()[triangle.i1];
                        let v2 = &self.data.get_vertices()[triangle.i2];
                        let v3 = &self.data.get_vertices()[triangle.i3];

                        let uv1 = v1.get_uv().as_ref().unwrap();
                        let uv2 = v2.get_uv().as_ref().unwrap();
                        let uv3 = v3.get_uv().as_ref().unwrap();

                        util::vec2_apply_bary_coord(uv1, uv2, uv3, &data.bary_coords)
                    }
                }
            }

            fn get_normal(&self, data: &DataForInterpolation) -> glm::DVec3 {
                match data.primitive_type {
                    PrimitiveType::Triangle => {
                        let triangle = &self.triangles.as_ref().expect("not cached yet")
                            [data.primitive_index.unwrap()];
                        let v1 = &self.data.get_vertices()[triangle.i1];
                        let v2 = &self.data.get_vertices()[triangle.i2];
                        let v3 = &self.data.get_vertices()[triangle.i3];

                        let normal1 = v1.get_normal().as_ref().unwrap();
                        let normal2 = v2.get_normal().as_ref().unwrap();
                        let normal3 = v3.get_normal().as_ref().unwrap();

                        util::vec3_apply_bary_coord(normal1, normal2, normal3, &data.bary_coords)
                    }
                }
            }

            fn set_cached_data(&mut self) {
                if self.triangles.is_none() {
                    self.triangles = Some(
                        self.data
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
                                        Triangle::new(v1_index, v2_index, v3_index)
                                    },
                                )
                            })
                            .collect_vec(),
                    );
                }
            }

            fn rebuild_bvh_if_needed(&mut self, epsilon: f64) {
                self.data.rebuild_bvh_if_needed(epsilon);
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    }
}
