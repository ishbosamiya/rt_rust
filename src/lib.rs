pub mod bvh;
pub mod fps;
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

pub use nalgebra_glm as glm;

use path_trace::{
    camera::Camera, environment::Environment, shader_list::ShaderList, texture_list::TextureList,
};
use scene::Scene;
use std::sync::{Arc, RwLock};

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
