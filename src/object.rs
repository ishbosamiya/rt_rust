use std::{cell::RefCell, rc::Rc};

use crate::{
    mesh::MeshDrawError,
    path_trace::{intersectable::Intersectable, shader_list::ShaderID as PathTraceShaderID},
    rasterize::{drawable::Drawable, gpu_immediate::GPUImmediate},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawError {
    Mesh(MeshDrawError),
    Sphere(()),
}

pub struct ObjectDrawData {
    imm: Rc<RefCell<GPUImmediate>>,
}

impl ObjectDrawData {
    pub fn new(imm: Rc<RefCell<GPUImmediate>>) -> Self {
        Self { imm }
    }
}

pub trait Object:
    Intersectable + Drawable<ExtraData = ObjectDrawData, Error = DrawError> + Sync
{
    fn set_path_trace_shader_id(&mut self, shader_id: PathTraceShaderID);
    fn get_path_trace_shader_id(&self) -> PathTraceShaderID;
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
                    outside_color,
                    inside_color,
                }
            }
        }

        impl Intersectable for Sphere {
            fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<IntersectInfo> {
                self.data.hit(ray, t_min, t_max)
            }
        }

        impl Drawable for Sphere {
            type ExtraData = ObjectDrawData;
            type Error = DrawError;

            fn draw(&self, extra_data: &mut ObjectDrawData) -> Result<(), DrawError> {
                self.data
                    .draw(&mut SphereDrawData::new(
                        extra_data.imm.clone(),
                        self.outside_color,
                        self.inside_color,
                    ))
                    .map_err(|_error| DrawError::Sphere(()))
            }

            fn draw_wireframe(&self, extra_data: &mut ObjectDrawData) -> Result<(), DrawError> {
                self.data
                    .draw_wireframe(&mut SphereDrawData::new(
                        extra_data.imm.clone(),
                        self.outside_color,
                        self.inside_color,
                    ))
                    .map_err(|_error| DrawError::Sphere(()))
            }
        }

        impl Object for Sphere {
            fn set_path_trace_shader_id(&mut self, shader_id: ShaderID) {
                self.shader_id = Some(shader_id)
            }

            fn get_path_trace_shader_id(&self) -> ShaderID {
                self.shader_id.unwrap()
            }
        }
    }

    mod mesh {
        use crate::{
            glm,
            mesh::{Mesh as MeshData, MeshDrawData, MeshUseShader},
            path_trace::{
                intersectable::{IntersectInfo, Intersectable},
                ray::Ray,
                shader_list::ShaderID,
            },
            rasterize::drawable::Drawable,
        };

        use super::super::{DrawError, Object, ObjectDrawData};

        pub struct Mesh {
            data: MeshData,
            shader_id: Option<ShaderID>,

            // TODO: since this is a partial copy of MeshDrawData, it
            // might make sense to store this in a separate structure and
            // use that
            use_shader: MeshUseShader,
            draw_bvh: bool,
            bvh_draw_level: usize,
            bvh_color: glm::DVec4,
        }

        impl Mesh {
            pub fn new(
                data: MeshData,
                use_shader: MeshUseShader,
                draw_bvh: bool,
                bvh_draw_level: usize,
                bvh_color: glm::DVec4,
            ) -> Self {
                Self {
                    data,
                    shader_id: None,

                    use_shader,
                    draw_bvh,
                    bvh_draw_level,
                    bvh_color,
                }
            }
        }

        impl Intersectable for Mesh {
            fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<IntersectInfo> {
                self.data.hit(ray, t_min, t_max)
            }
        }

        impl Drawable for Mesh {
            type ExtraData = ObjectDrawData;
            type Error = DrawError;

            fn draw(&self, extra_data: &mut ObjectDrawData) -> Result<(), DrawError> {
                self.data
                    .draw(&mut MeshDrawData::new(
                        extra_data.imm.clone(),
                        self.use_shader,
                        self.draw_bvh,
                        self.bvh_draw_level,
                        self.bvh_color,
                    ))
                    .map_err(DrawError::Mesh)
            }

            fn draw_wireframe(&self, extra_data: &mut ObjectDrawData) -> Result<(), DrawError> {
                self.data
                    .draw_wireframe(&mut MeshDrawData::new(
                        extra_data.imm.clone(),
                        MeshUseShader::DirectionalLight,
                        self.draw_bvh,
                        self.bvh_draw_level,
                        self.bvh_color,
                    ))
                    .map_err(DrawError::Mesh)
            }
        }

        impl Object for Mesh {
            fn set_path_trace_shader_id(&mut self, shader_id: ShaderID) {
                self.shader_id = Some(shader_id)
            }

            fn get_path_trace_shader_id(&self) -> ShaderID {
                self.shader_id.unwrap()
            }
        }
    }
}
