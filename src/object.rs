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

pub struct ObjectDrawData<'a> {
    imm: &'a mut GPUImmediate,
}

impl<'a> ObjectDrawData<'a> {
    pub fn new(imm: &'a mut GPUImmediate) -> Self {
        Self { imm }
    }

    pub fn get_imm(&mut self) -> &mut GPUImmediate {
        self.imm
    }
}

pub trait Object<'a>: Intersectable + Drawable<'a> + Sync {
    fn set_path_trace_shader_id(&mut self, shader_id: PathTraceShaderID);
    fn get_path_trace_shader_id(&self) -> PathTraceShaderID;
}

pub mod objects {
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

    use super::{DrawError, Object, ObjectDrawData};

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
        pub fn new(data: SphereData, outside_color: glm::Vec4, inside_color: glm::Vec4) -> Self {
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

    impl<'a> Drawable<'a> for Sphere {
        type ExtraData = ObjectDrawData<'a>;
        type Error = DrawError;

        fn draw(&self, extra_data: &mut ObjectDrawData<'_>) -> Result<(), DrawError> {
            (&self.data)
                .draw(&mut SphereDrawData::new(
                    extra_data.get_imm(),
                    self.outside_color,
                    self.inside_color,
                ))
                .map_err(|_error| DrawError::Sphere(()))
        }

        fn draw_wireframe(&self, extra_data: &mut ObjectDrawData<'_>) -> Result<(), DrawError> {
            (&self.data)
                .draw_wireframe(&mut SphereDrawData::new(
                    extra_data.imm,
                    self.outside_color,
                    self.inside_color,
                ))
                .map_err(|_error| DrawError::Sphere(()))
        }
    }

    impl Object<'_> for Sphere {
        fn set_path_trace_shader_id(&mut self, shader_id: ShaderID) {
            self.shader_id = Some(shader_id)
        }

        fn get_path_trace_shader_id(&self) -> ShaderID {
            self.shader_id.unwrap()
        }
    }
}
