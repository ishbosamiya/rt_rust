use std::collections::{hash_map, HashMap};

use quick_renderer::{rasterize::Rasterize, texture::TextureRGBAFloat};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

use crate::{egui, ui::DrawUI, UiData};

/// A unique identifier given to each `Texture` during its
/// initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TextureID(usize);

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureList {
    /// list of all textures indexed by their TextureID
    textures: HashMap<TextureID, TextureRGBAFloat>,
    /// list of all texture ids in the order of addition of textures
    texture_ids: Vec<TextureID>,
}

impl TextureList {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            texture_ids: Vec::new(),
        }
    }

    /// Load texture from disk given the path to the texture
    pub fn load_texture<P>(&mut self, path: P)
    where
        P: AsRef<std::path::Path>,
    {
        self.add_texture(TextureRGBAFloat::load_from_disk(path).unwrap());
    }

    /// Load texture from disk with file dialog to choose the texture
    pub fn load_texture_with_file_dialog(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("png", &["png"])
            .add_filter("jpg", &["jpg", "jpeg"])
            .add_filter("tiff", &["tiff"])
            .add_filter("Any", &["*"])
            .set_directory(".")
            .pick_file()
        {
            self.load_texture(path);
        }
    }

    pub fn get_textures(&self) -> hash_map::Iter<'_, TextureID, TextureRGBAFloat> {
        self.textures.iter()
    }

    pub fn get_textures_mut(&mut self) -> hash_map::IterMut<'_, TextureID, TextureRGBAFloat> {
        self.textures.iter_mut()
    }

    pub fn get_texture(&self, texture_id: TextureID) -> Option<&TextureRGBAFloat> {
        self.textures.get(&texture_id)
    }

    pub fn get_texture_mut(&mut self, texture_id: TextureID) -> Option<&mut TextureRGBAFloat> {
        self.textures.get_mut(&texture_id)
    }

    pub fn add_texture(&mut self, texture: TextureRGBAFloat) -> TextureID {
        let texture_id = TextureID(rand::random());
        self.textures.insert(texture_id, texture);
        self.texture_ids.push(texture_id);
        texture_id
    }

    pub fn delete_texture(&mut self, texture_id: TextureID) {
        self.texture_ids.remove(
            self.texture_ids
                .iter()
                .enumerate()
                .find(|(_, id)| texture_id == **id)
                .unwrap()
                .0,
        );

        self.textures.remove(&texture_id).unwrap();
    }

    /// Get a reference to the texture list's texture ids.
    pub fn get_texture_ids(&self) -> &[TextureID] {
        self.texture_ids.as_slice()
    }
}

impl Default for TextureList {
    fn default() -> Self {
        Self::new()
    }
}

impl Rasterize for TextureList {
    fn cleanup_opengl(&mut self) {
        self.get_textures_mut()
            .for_each(|(_, texture)| texture.cleanup_opengl());
    }
}

impl DrawUI for TextureList {
    type ExtraData = UiData;

    fn draw_ui(&self, _ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {}

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {
        assert_eq!(self.texture_ids.len(), self.textures.len());

        ui.collapsing("Texture List", |ui| {
            let texture_width = ui.available_width();

            let mut delete_texture = None;
            let textures = &mut self.textures;
            self.texture_ids
                .iter()
                .enumerate()
                .for_each(|(index, texture_id)| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("Texture: {}", index + 1));
                        if ui.button("X").clicked() {
                            delete_texture = Some(*texture_id);
                        }
                    });

                    let texture = textures.get_mut(texture_id).unwrap();
                    ui.image(
                        egui::TextureId::User(texture.get_gl_tex().into()),
                        &[
                            texture_width,
                            texture_width * texture.get_height() as f32
                                / texture.get_width() as f32,
                        ],
                    );
                });

            if let Some(texture_id) = delete_texture {
                self.delete_texture(texture_id);
            }

            if ui.button("Load Texture").clicked() {
                self.load_texture_with_file_dialog();
            }
        });
    }
}
