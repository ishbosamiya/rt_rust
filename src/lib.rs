pub mod blend;
pub mod bvh;
pub mod camera;
#[cfg(feature = "use_embree")]
pub mod embree;
pub mod file;
pub mod fps;
pub mod icons;
pub mod image;
pub mod inputs;
pub mod math;
pub mod mesh;
pub mod meshio;
pub mod namegen;
pub mod object;
pub mod path_trace;
pub mod progress;
pub mod rasterize;
pub mod scene;
pub mod sphere;
pub mod transform;
pub mod ui;
pub mod util;
pub mod viewport;

pub(crate) use egui_glfw::egui;

extern crate image as external_image;

use mesh::MeshUseShader;
use meshio::MeshIO;
pub use nalgebra_glm as glm;

use camera::Camera;
use object::Object;
use path_trace::{environment::Environment, shader_list::ShaderList, texture_list::TextureList};
use scene::Scene;
use std::{
    convert::TryInto,
    sync::{Arc, RwLock},
};

pub struct UiData {
    scene: Arc<RwLock<Scene>>,
    shader_list: Arc<RwLock<ShaderList>>,
    texture_list: Arc<RwLock<TextureList>>,
    camera: Arc<RwLock<Camera>>,
    environment: Arc<RwLock<Environment>>,
}

impl UiData {
    pub fn new(
        scene: Arc<RwLock<Scene>>,
        shader_list: Arc<RwLock<ShaderList>>,
        texture_list: Arc<RwLock<TextureList>>,
        camera: Arc<RwLock<Camera>>,
        environment: Arc<RwLock<Environment>>,
    ) -> Self {
        Self {
            scene,
            shader_list,
            texture_list,
            camera,
            environment,
        }
    }

    /// Get a reference to the ui data's scene.
    pub fn get_scene(&self) -> &Arc<RwLock<Scene>> {
        &self.scene
    }

    /// Get a reference to the ui data's shader list.
    pub fn get_shader_list(&self) -> &Arc<RwLock<ShaderList>> {
        &self.shader_list
    }

    /// Get a reference to the ui data's texture list.
    pub fn get_texture_list(&self) -> &Arc<RwLock<TextureList>> {
        &self.texture_list
    }

    /// Get a reference to the ui data's camera.
    pub fn get_camera(&self) -> &Arc<RwLock<Camera>> {
        &self.camera
    }

    /// Get a reference to the ui data's environment.
    pub fn get_environment(&self) -> &Arc<RwLock<Environment>> {
        &self.environment
    }
}

pub fn default_image_width() -> usize {
    200
}

pub fn default_image_height() -> usize {
    200
}

pub fn default_samples_per_pixel() -> usize {
    5
}

pub fn default_trace_max_depth() -> usize {
    5
}

pub fn default_environment_strength() -> f64 {
    1.0
}

use crate::object::objects::Mesh as MeshObject;

pub fn load_obj_file<P>(path: P) -> Vec<MeshObject>
where
    P: AsRef<std::path::Path>,
{
    // TODO: Handle errors throught the process

    // read the file
    let meshio = MeshIO::read(path).unwrap();
    // split the meshio based on
    // objects and create mesh(s) from
    // meshio(s)
    meshio
        .split()
        .drain(0..)
        .map(|meshio| {
            let mesh = crate::mesh::Mesh::read(&meshio).unwrap();
            // add mesh to scene
            let mut object = MeshObject::new(
                mesh,
                MeshUseShader::DirectionalLight {
                    color: util::srgb_to_linear(&glm::vec3(0.3, 0.2, 0.7)),
                },
                None,
            );
            object.set_model_matrix(glm::identity());
            if let Some(name) = meshio.object_names.first().unwrap() {
                *object.get_object_name_mut() = name.to_string();
            }
            object
        })
        .collect()
}

/// Save image to disk, based on the extension picks the correct file format.
///
/// # Note
///
/// `linear_to_srgb` conversion is not done when saving to `image`
/// file format (the custom file format)
pub fn save_image<P>(image: &image::Image, linear_to_srgb: bool, path: P)
where
    P: AsRef<std::path::Path>,
{
    let save_to_generic_format = || {
        let image = external_image::ImageBuffer::from_fn(
            image.width().try_into().unwrap(),
            image.height().try_into().unwrap(),
            |i, j| {
                let pixel = image.get_pixel(i.try_into().unwrap(), j.try_into().unwrap());
                let pixel = [pixel[0] as f32, pixel[1] as f32, pixel[2] as f32];

                let pixel = if linear_to_srgb {
                    [
                        egui::color::gamma_from_linear(pixel[0]),
                        egui::color::gamma_from_linear(pixel[1]),
                        egui::color::gamma_from_linear(pixel[2]),
                    ]
                } else {
                    pixel
                };

                let pixel = [
                    (pixel[0] * 255.0).round(),
                    (pixel[1] * 255.0).round(),
                    (pixel[2] * 255.0).round(),
                    255.0,
                ];

                external_image::Rgba([
                    pixel[0] as u8,
                    pixel[1] as u8,
                    pixel[2] as u8,
                    pixel[3] as u8,
                ])
            },
        );

        image.save(&path).unwrap();
    };

    let save_to_custom_format = || {
        let file = serde_json::to_string(image).unwrap();
        std::fs::write(&path, file).unwrap();
    };

    if let Some(extension) = path.as_ref().extension() {
        if extension == "image" {
            save_to_custom_format();
        } else {
            save_to_generic_format();
        }
    } else {
        save_to_custom_format();
    }
}
