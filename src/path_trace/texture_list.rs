use std::collections::{hash_map, HashMap};
use std::convert::TryInto;

use image::GenericImageView;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

use crate::glm;
use crate::{rasterize::texture::TextureRGBAFloat, ui::DrawUI};

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

    pub fn get_textures(&self) -> hash_map::Values<'_, TextureID, TextureRGBAFloat> {
        self.textures.values()
    }

    pub fn get_texture(&self, texture_id: TextureID) -> Option<&TextureRGBAFloat> {
        self.textures.get(&texture_id)
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
}

impl Default for TextureList {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawUI for TextureList {
    fn draw_ui(&self, _ui: &mut egui::Ui) {}

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui) {
        assert_eq!(self.texture_ids.len(), self.textures.len());

        ui.collapsing("Texture List", |ui| {
            let texture_width = ui.available_width();

            let mut delete_texture = None;
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

                    let texture = self.textures.get(texture_id).unwrap();
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
                if let Some(path) = FileDialog::new()
                    .add_filter("png", &["png"])
                    .add_filter("jpg", &["jpg", "jpeg"])
                    .add_filter("tiff", &["tiff"])
                    .add_filter("Any", &["*"])
                    .set_directory(".")
                    .pick_file()
                {
                    let file = std::fs::File::open(path).unwrap();
                    let image_reader = image::io::Reader::new(std::io::BufReader::new(file))
                        .with_guessed_format()
                        .unwrap();
                    let image = image_reader.decode().unwrap();
                    let texture = TextureRGBAFloat::from_pixels(
                        image.width().try_into().unwrap(),
                        image.height().try_into().unwrap(),
                        image
                            .to_rgba16()
                            .pixels()
                            .map(|pixel| {
                                glm::vec4(
                                    pixel[0] as f32 / u16::MAX as f32,
                                    pixel[1] as f32 / u16::MAX as f32,
                                    pixel[2] as f32 / u16::MAX as f32,
                                    pixel[3] as f32 / u16::MAX as f32,
                                )
                            })
                            .collect(),
                    );

                    self.add_texture(texture);
                }
            }
        });
    }
}
