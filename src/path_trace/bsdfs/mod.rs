pub mod blinnphong;
pub mod emissive;
pub mod glossy;
pub mod lambert;
pub mod refraction;
pub mod utils;

use std::sync::{Arc, RwLock};

use super::texture_list::TextureList;

pub struct BSDFUiData {
    texture_list: Arc<RwLock<TextureList>>,
    shader_egui_id: egui::Id,
}

impl BSDFUiData {
    pub fn new(texture_list: Arc<RwLock<TextureList>>, shader_egui_id: egui::Id) -> Self {
        Self {
            texture_list,
            shader_egui_id,
        }
    }

    /// Get a reference to the texture list.
    pub fn get_texture_list(&self) -> &Arc<RwLock<TextureList>> {
        &self.texture_list
    }

    /// Get a reference to the shader egui id.
    pub fn get_shader_egui_id(&self) -> &egui::Id {
        &self.shader_egui_id
    }
}
