use std::{cell::RefCell, rc::Rc};

use crate::{
    glm,
    mesh::MeshDrawError,
    path_trace::{intersectable::Intersectable, shader_list::ShaderID as PathTraceShaderID},
    rasterize::{drawable::Drawable, gpu_immediate::GPUImmediate},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawError {
    Mesh(MeshDrawError),
    Sphere(()),
}

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
}

impl ObjectDrawData {
    pub fn new(imm: Rc<RefCell<GPUImmediate>>) -> Self {
        Self {
            imm,
            use_model_matrix: true,
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
}

pub trait Object:
    Intersectable + Drawable<ExtraData = ObjectDrawData, Error = DrawError> + Sync + Send
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
    fn get_path_trace_shader_id(&self) -> PathTraceShaderID;

    fn get_min_max_bounds(&self) -> (glm::DVec3, glm::DVec3);
}

pub mod objects {
    pub use mesh::Mesh;
    pub use sphere::Sphere;

    mod sphere {
        use crate::{
            glm,
            path_trace::{
                intersectable::{IntersectInfo, Intersectable},
                ray::Ray,
                shader_list::ShaderID,
            },
            rasterize::drawable::Drawable,
            sphere::{Sphere as SphereData, SphereDrawData},
        };

        use super::super::{DrawError, Object, ObjectDrawData};

        pub struct Sphere {
            data: SphereData,
            shader_id: Option<ShaderID>,
            model_matrix: Option<glm::DMat4>,

            // TODO: since this is a partial copy of SphereDrawData, it
            // might make sense to store this in a separate structure and
            // use that
            outside_color: glm::Vec4,
            inside_color: glm::Vec4,
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
                    info
                })
            }
        }

        impl Drawable for Sphere {
            type ExtraData = ObjectDrawData;
            type Error = DrawError;

            fn draw(&self, extra_data: &mut ObjectDrawData) -> Result<(), DrawError> {
                let model = if extra_data.use_model_matrix {
                    self.get_model_matrix().unwrap()
                } else {
                    glm::identity()
                };

                self.data
                    .draw(&mut SphereDrawData::new(
                        extra_data.imm.clone(),
                        model,
                        self.outside_color,
                        self.inside_color,
                    ))
                    .map_err(|_error| DrawError::Sphere(()))?;

                Ok(())
            }

            fn draw_wireframe(&self, extra_data: &mut ObjectDrawData) -> Result<(), DrawError> {
                let model = if extra_data.use_model_matrix {
                    self.get_model_matrix().unwrap()
                } else {
                    glm::identity()
                };

                self.data
                    .draw_wireframe(&mut SphereDrawData::new(
                        extra_data.imm.clone(),
                        model,
                        self.outside_color,
                        self.inside_color,
                    ))
                    .map_err(|_error| DrawError::Sphere(()))
            }
        }

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

            fn get_path_trace_shader_id(&self) -> ShaderID {
                self.shader_id.unwrap()
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
        }
    }

    mod mesh {
        use crate::{
            glm,
            mesh::{Mesh as MeshData, MeshBVHDrawData, MeshDrawData, MeshUseShader},
            path_trace::{
                intersectable::{IntersectInfo, Intersectable},
                ray::Ray,
                shader_list::ShaderID,
            },
            rasterize::{drawable::Drawable, shader},
        };

        use super::super::{DrawError, Object, ObjectDrawData};

        pub struct Mesh {
            data: MeshData,
            shader_id: Option<ShaderID>,
            model_matrix: Option<glm::DMat4>,

            // TODO: since this is a partial copy of MeshDrawData, it
            // might make sense to store this in a separate structure and
            // use that
            use_shader: MeshUseShader,
            bvh_draw_data: Option<MeshBVHDrawData>,
        }

        impl Mesh {
            pub fn new(
                data: MeshData,
                use_shader: MeshUseShader,
                bvh_draw_data: Option<MeshBVHDrawData>,
            ) -> Self {
                Self {
                    data,
                    shader_id: None,
                    model_matrix: None,

                    use_shader,
                    bvh_draw_data,
                }
            }
        }

        impl Intersectable for Mesh {
            fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<IntersectInfo> {
                self.data.hit(ray, t_min, t_max).map(|mut info| {
                    info.set_shader_id(self.get_path_trace_shader_id());
                    info
                })
            }
        }

        impl Drawable for Mesh {
            type ExtraData = ObjectDrawData;
            type Error = DrawError;

            fn draw(&self, extra_data: &mut ObjectDrawData) -> Result<(), DrawError> {
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
                    .draw(&mut MeshDrawData::new(
                        extra_data.imm.clone(),
                        self.use_shader,
                        self.bvh_draw_data,
                    ))
                    .map_err(DrawError::Mesh)
            }

            fn draw_wireframe(&self, _extra_data: &mut ObjectDrawData) -> Result<(), DrawError> {
                todo!()
            }
        }

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

            fn get_path_trace_shader_id(&self) -> ShaderID {
                self.shader_id.unwrap()
            }

            fn get_min_max_bounds(&self) -> (glm::DVec3, glm::DVec3) {
                self.data.get_min_max_bounds()
            }
        }
    }
}
