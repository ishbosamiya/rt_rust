use rfd::FileDialog;
use serde::{Deserialize, Serialize};

use crate::{image::Image, transform::Transform, ui::DrawUI};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    hdr: Image,
    strength: f64,

    #[serde(default = "default_transform")]
    transform: Transform,
}

fn default_transform() -> Transform {
    Transform::default()
}

impl Default for Environment {
    fn default() -> Self {
        Self::new(Image::new(4, 4), 1.0, Transform::default())
    }
}

impl Environment {
    pub fn new(hdr: Image, strength: f64, transform: Transform) -> Self {
        Self {
            hdr,
            strength,
            transform,
        }
    }

    /// Get a reference to the environment's hdr.
    pub fn get_hdr(&self) -> &Image {
        &self.hdr
    }

    /// Get environment's strength.
    pub fn get_strength(&self) -> f64 {
        self.strength
    }

    /// Get a reference to the environment's transform.
    pub fn get_transform(&self) -> &Transform {
        &self.transform
    }

    /// Set the environment's strength.
    pub fn set_strength(&mut self, strength: f64) {
        self.strength = strength;
    }

    /// Set the environment's hdr.
    pub fn set_hdr(&mut self, hdr: Image) {
        self.hdr = hdr;
    }

    /// Set the environment's transform.
    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }
}

impl DrawUI for Environment {
    fn draw_ui(&self, _ui: &mut egui::Ui) {}

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.strength, 0.0..=5.0).text("Environment Strength"));

        if ui.button("Load Environment Image").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("HDR", &["hdr"])
                .add_filter("Any", &["*"])
                .set_directory(".")
                .pick_file()
            {
                let hdr = image::codecs::hdr::HdrDecoder::new(std::io::BufReader::new(
                    std::fs::File::open(path).unwrap(),
                ))
                .unwrap();
                let width = hdr.metadata().width as _;
                let height = hdr.metadata().height as _;
                self.hdr = Image::from_vec_rgb_f32(&hdr.read_image_hdr().unwrap(), width, height);
            }
        }

        self.transform.draw_ui(ui);
        self.transform.draw_ui_mut(ui);
    }
}

pub struct EnvironmentShadingData<'a> {
    hdr: &'a Image,
    strength: f64,
    transform: &'a Transform,
}

impl<'a> EnvironmentShadingData<'a> {
    /// # Safety
    ///
    /// In most instances, this structure should be created from
    /// [`Environment`] instead of creating it manually. It is as
    /// simple as `environment.into()`.
    ///
    pub unsafe fn new(hdr: &'a Image, strength: f64, transform: &'a Transform) -> Self {
        Self {
            hdr,
            strength,
            transform,
        }
    }

    /// Get a reference to the environment hdr.
    pub fn get_hdr(&self) -> &'a Image {
        self.hdr
    }

    /// Get environment strength.
    pub fn get_strength(&self) -> f64 {
        self.strength
    }

    /// Get environment transform.
    pub fn get_transform(&self) -> &'a Transform {
        self.transform
    }
}

impl<'a> From<&'a Environment> for EnvironmentShadingData<'a> {
    fn from(env: &'a Environment) -> Self {
        unsafe { Self::new(env.get_hdr(), env.get_strength(), env.get_transform()) }
    }
}
