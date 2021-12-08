pub mod blinnphong;
pub mod debug_bsdf;
pub mod emissive;
pub mod glass;
pub mod glass_dispersion;
pub mod glossy;
pub mod lambert;
pub mod refraction;
pub mod refraction_dispersion;
pub mod utils;

use std::sync::{Arc, RwLock};

use super::texture_list::TextureList;
use crate::egui;

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
