use std::fmt::Display;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::path_trace::texture_list::{TextureID, TextureList};
use crate::ui::DrawUI;
use crate::{egui, glm, math, ui};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColorPicker {
    Color(glm::DVec3),
    Texture(Option<TextureID>),
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self::Color(glm::vec3(1.0, 1.0, 1.0))
    }
}

impl Display for ColorPicker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorPicker::Color(_) => write!(f, "Color"),
            ColorPicker::Texture(_) => write!(f, "Texture"),
        }
    }
}

impl ColorPicker {
    /// Get color value from [`ColorPicker`], returns `None` if
    /// texture id is None or texture is not available in the texture
    /// list.
    pub fn get_color_checked(
        &self,
        uv: &glm::DVec2,
        texture_list: &TextureList,
    ) -> Option<glm::DVec3> {
        match self {
            ColorPicker::Color(color) => Some(*color),
            ColorPicker::Texture(texture_id) => {
                if let Some(texture_id) = texture_id {
                    let pixel = texture_list.get_texture(*texture_id)?.get_pixel_uv(uv);
                    Some(glm::vec3(pixel[0].into(), pixel[1].into(), pixel[2].into()))
                } else {
                    None
                }
            }
        }
    }

    /// Get color value from [`ColorPicker`]. Return (1.0, 0.0, 1.0)
    /// if unable to fetch the color.
    ///
    /// Ideally, using [`ColorPicker::get_color_checked()`] is the way
    /// to go but in most instances the error is not propagated
    /// forward, so the same `self.get_color_checked(uv,
    /// texture_list).unwrap_or_else(|| glm::vec3(1.0, 0.0, 1.0))` is
    /// done each time. This makes that a lot more convinent.
    pub fn get_color(&self, uv: &glm::DVec2, texture_list: &TextureList) -> glm::DVec3 {
        self.get_color_checked(uv, texture_list)
            .unwrap_or_else(|| glm::vec3(1.0, 0.0, 1.0))
    }
}

pub struct ColorPickerUiData {
    texture_list: Arc<RwLock<TextureList>>,
    color_picker_id: egui::Id,
}

impl ColorPickerUiData {
    pub fn new(texture_list: Arc<RwLock<TextureList>>, color_picker_id: egui::Id) -> Self {
        Self {
            texture_list,
            color_picker_id,
        }
    }
}

impl DrawUI for ColorPicker {
    type ExtraData = ColorPickerUiData;

    fn draw_ui(&self, _ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {}

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui, extra_data: &Self::ExtraData) {
        egui::ComboBox::from_id_source(extra_data.color_picker_id)
            .selected_text(format!("{}", self))
            .show_ui(ui, |ui| {
                ui.selectable_value(self, ColorPicker::Color(glm::vec3(1.0, 1.0, 1.0)), "Color");
                ui.selectable_value(self, ColorPicker::Texture(None), "Texture");
            });

        match self {
            ColorPicker::Color(color) => ui::color_edit_button_dvec3(ui, "", color),
            ColorPicker::Texture(selected_texture_id) => {
                ui.vertical(|ui| {
                    if let Ok(mut texture_list) = extra_data.texture_list.try_write() {
                        if let Some(texture) = selected_texture_id
                            .and_then(|texture_id| texture_list.get_texture_mut(texture_id))
                        {
                            ui.label("Selected Texture:");
                            ui.image(
                                egui::TextureId::User(texture.get_gl_tex().into()),
                                &[
                                    150.0,
                                    150.0 * texture.get_height() as f32
                                        / texture.get_width() as f32,
                                ],
                            );
                        } else {
                            ui.label("No Texture Selected");
                        }
                        egui::CollapsingHeader::new("Select Texture")
                            .id_source(extra_data.color_picker_id.with("Select Texture"))
                            .show(ui, |ui| {
                                texture_list.get_textures_mut().for_each(
                                    |(texture_id, texture)| {
                                        ui.horizontal(|ui| {
                                            if ui.button(".").clicked() {
                                                *selected_texture_id = Some(*texture_id);
                                            }
                                            ui.image(
                                                egui::TextureId::User(texture.get_gl_tex().into()),
                                                &[
                                                    100.0,
                                                    100.0 * texture.get_height() as f32
                                                        / texture.get_width() as f32,
                                                ],
                                            );
                                        });
                                    },
                                );
                            });
                    } else {
                        ui.label(
                            "Textures not available right now, currently accessed by something",
                        );
                    }
                });
            }
        }
    }
}

/// Get `wi` when sampling pure diffuse
pub fn wi_diffuse(normal: &glm::DVec3) -> glm::DVec3 {
    // TODO: make this random in hemisphere instead of using a
    // sphere for better performance

    //need to return `wi` which should point towards the hitpoint
    -(normal + math::random_in_unit_sphere())
}
