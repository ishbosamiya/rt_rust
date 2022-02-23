use quick_renderer::texture::TextureRGBAFloat;

use crate::{glm, image::Image};

pub trait TextureRGBAFloatExtension {
    fn from_image(tex: &Image) -> Self;
    fn update_from_image(&mut self, tex: &Image);
}

impl TextureRGBAFloatExtension for TextureRGBAFloat {
    fn from_image(tex: &Image) -> Self {
        let mut res = Self::from_pixels(
            tex.width(),
            tex.height(),
            tex.get_pixels()
                .chunks(tex.width())
                .rev()
                .flat_map(|row| {
                    row.iter().map(|pixel| {
                        glm::vec4(pixel[0] as f32, pixel[1] as f32, pixel[2] as f32, 1.0)
                    })
                })
                .collect(),
        );
        unsafe {
            res.set_id(tex.get_id());
        }
        res
    }

    fn update_from_image(&mut self, tex: &Image) {
        // If the ids are the same, the pixels are also the same so
        // don't do anything
        if self.get_id() == tex.get_id() {
            return;
        }

        *self = Self::from_image(tex);
    }
}
