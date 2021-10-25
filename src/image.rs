use std::fs::File;
use std::io::prelude::*;

use crate::glm;
use crate::rasterize::texture::TextureRGBAFloat;

#[derive(Debug, Clone)]
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
                .map(|(r, g, b, _a)| glm::vec3(*r as f64, *g as f64, *b as f64))
                .collect(),
            width: tex.get_width(),
            height: tex.get_height(),
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn set_pixel(&mut self, i: usize, j: usize, data: glm::DVec3) {
        self.id = rand::random();
        self.pixels[i * self.width + j] = data;
    }

    pub fn get_pixel(&self, i: usize, j: usize) -> &glm::DVec3 {
        &self.pixels[i * self.width + j]
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

pub struct PPM<'a> {
    image: &'a Image,
}

impl PPM<'_> {
    pub fn new(image: &Image) -> PPM {
        PPM { image }
    }

    pub fn write_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        let mut string_data = String::new();

        let header = "P3\n";
        string_data.push_str(header);

        let sizing = format!("{} {}\n", self.image.width(), self.image.height());
        string_data.push_str(&sizing);

        let max_val = "255\n";
        string_data.push_str(max_val);

        for i in self.image.get_pixels().chunks(self.image.width()) {
            for j in i {
                let j = glm::clamp(j, 0.0, 1.0);
                string_data.push_str(&((j[0] * 255.0) as i64 % 256).to_string());
                string_data.push(' ');
                string_data.push_str(&((j[1] * 255.0) as i64 % 256).to_string());
                string_data.push(' ');
                string_data.push_str(&((j[2] * 255.0) as i64 % 256).to_string());
                string_data.push(' ');
            }
            string_data.push('\n');
        }

        let mut fout = File::create(path).unwrap();
        fout.write_all(string_data.as_bytes())?;

        Ok(())
    }
}
