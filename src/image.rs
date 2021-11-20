use image::Pixel;
use serde::{Deserialize, Serialize};

use crate::glm;
use crate::rasterize::texture::TextureRGBAFloat;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    /// If the id has changed, the pixel data might have also
    /// changed. Every time the image is borrowed mutably, the id is
    /// update.
    id: usize,

    pixels: Vec<glm::DVec3>,
    width: usize,
    height: usize,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Image {
        let mut pixels: Vec<glm::DVec3> = Vec::with_capacity(width * height);
        pixels.resize(width * height, glm::vec3(0.0, 0.0, 0.0));

        Self::from_pixels(width, height, pixels)
    }

    pub fn from_pixels(width: usize, height: usize, pixels: Vec<glm::DVec3>) -> Image {
        Image {
            id: rand::random(),
            pixels,
            width,
            height,
        }
    }

    pub fn from_texture_rgba_float(tex: &TextureRGBAFloat) -> Image {
        Self {
            id: rand::random(),
            pixels: tex
                .get_pixels()
                .iter()
                .map(|pixel| glm::vec3(pixel[0].into(), pixel[1].into(), pixel[2].into()))
                .collect(),
            width: tex.get_width(),
            height: tex.get_height(),
        }
    }

    pub fn from_vec_rgb_f32(pixels: &[image::Rgb<f32>], width: usize, height: usize) -> Image {
        Self {
            id: rand::random(),
            pixels: pixels
                .iter()
                .map(|rgb| {
                    glm::vec3(
                        rgb.channels()[0] as f64,
                        rgb.channels()[1] as f64,
                        rgb.channels()[2] as f64,
                    )
                })
                .collect(),
            width,
            height,
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn set_pixel(&mut self, i: usize, j: usize, data: glm::DVec3) {
        self.id = rand::random();
        self.pixels[j * self.width + i] = data;
    }

    pub fn get_pixel(&self, i: usize, j: usize) -> &glm::DVec3 {
        &self.pixels[j * self.width + i]
    }

    /// Get the pixel from the specified UV coordinates
    ///
    /// Wrapping mode is set to repeat. TODO: need to make wrapping
    /// mode user definable
    ///
    /// UV bottom left is (0.0, 0.0) and top right is (1.0, 1.0), same
    /// as OpenGL
    pub fn get_pixel_uv(&self, uv: &glm::DVec2) -> &glm::DVec3 {
        let uv = glm::vec2(uv[0] % 1.0, uv[1] % 1.0);

        self.get_pixel(
            (uv[0] * self.width as f64) as _,
            self.height - (uv[1] * self.height as f64) as usize - 1,
        )
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get_pixels_mut(&mut self) -> &mut Vec<glm::DVec3> {
        self.id = rand::random();
        &mut self.pixels
    }

    pub fn get_pixels(&self) -> &Vec<glm::DVec3> {
        &self.pixels
    }

    pub fn get_slabs(&self, num_slabs: usize) -> Vec<Slab> {
        let width = self.width / num_slabs;
        let height = self.height;

        let mut slabs = Vec::new();
        for i in 0..num_slabs {
            slabs.push(Slab::new(i * width, 0, width, height));
        }
        let last_slab_width = self.width % num_slabs;
        let last_slab_height = self.height;
        slabs.push(Slab::new(
            num_slabs * width,
            0,
            last_slab_width,
            last_slab_height,
        ));

        slabs
    }
}

#[derive(Debug)]
pub struct Slab {
    pub x_start: usize,
    pub y_start: usize,
    pub width: usize,
    pub height: usize,
    pixels: Vec<Vec<glm::DVec3>>,
}

impl Slab {
    pub fn new(x_start: usize, y_start: usize, width: usize, height: usize) -> Self {
        Self {
            x_start,
            y_start,
            width,
            height,
            pixels: Vec::new(),
        }
    }

    pub fn set_pixels(&mut self, pixels: Vec<Vec<glm::DVec3>>) {
        self.pixels = pixels;
    }

    pub fn get_pixels(&self) -> &Vec<Vec<glm::DVec3>> {
        &self.pixels
    }
}
