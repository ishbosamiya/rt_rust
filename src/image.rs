use std::fs::File;
use std::io::prelude::*;

use nalgebra_glm as glm;

use crate::math::Vec3;

pub struct Image {
    pixels: Vec<Vec<Vec3>>,
    width: usize,
    height: usize,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Image {
        let mut pixels: Vec<Vec<Vec3>> = Vec::with_capacity(height);
        let mut empty_row = Vec::with_capacity(width);
        empty_row.resize(width, Vec3::new(0.0, 0.0, 0.0));
        pixels.resize(height, empty_row);

        return Image {
            pixels,
            width,
            height,
        };
    }

    pub fn set_pixel(&mut self, i: usize, j: usize, data: Vec3) {
        self.pixels[i][j] = data;
    }

    pub fn get_pixel(&self, i: usize, j: usize) -> &Vec3 {
        return &self.pixels[i][j];
    }

    pub fn width(&self) -> usize {
        return self.width;
    }

    pub fn height(&self) -> usize {
        return self.height;
    }

    pub fn get_pixels_mut(&mut self) -> &mut Vec<Vec<Vec3>> {
        return &mut self.pixels;
    }

    pub fn get_pixels(&self) -> &Vec<Vec<Vec3>> {
        return &self.pixels;
    }

    pub fn get_slabs(&self, num_slabs: usize) -> Vec<Slab> {
        let width = self.width / num_slabs;
        let height = self.height;

        let mut slabs = Vec::new();
        for i in 0..num_slabs {
            slabs.push(Slab::new(i * width, 0, width, height));
        }
        let last_slab_width = self.width % num_slabs;
        let last_slab_height = self.width;
        slabs.push(Slab::new(
            num_slabs * width,
            0,
            last_slab_width,
            last_slab_height,
        ));

        return slabs;
    }
}

pub struct Slab {
    pub x_start: usize,
    pub y_start: usize,
    pub width: usize,
    pub height: usize,
    pixels: Vec<Vec<Vec3>>,
}

impl Slab {
    pub fn new(x_start: usize, y_start: usize, width: usize, height: usize) -> Self {
        return Self {
            x_start,
            y_start,
            width,
            height,
            pixels: Vec::new(),
        };
    }

    pub fn set_pixels(&mut self, pixels: Vec<Vec<Vec3>>) {
        self.pixels = pixels;
    }

    pub fn get_pixels(&self) -> &Vec<Vec<Vec3>> {
        return &self.pixels;
    }
}

pub struct PPM<'a> {
    image: &'a Image,
}

impl PPM<'_> {
    pub fn new(image: &Image) -> PPM {
        return PPM { image };
    }

    pub fn write_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        let mut string_data = String::new();

        let header = "P3\n";
        string_data.push_str(header);

        let sizing = format!("{} {}\n", self.image.width(), self.image.height());
        string_data.push_str(&sizing);

        let max_val = "255\n";
        string_data.push_str(max_val);

        for i in self.image.get_pixels() {
            for j in i {
                let j = glm::clamp(&j, 0.0, 1.0);
                string_data.push_str(&((j[0] * 255.0) as i64 % 256).to_string());
                string_data.push_str(" ");
                string_data.push_str(&((j[1] * 255.0) as i64 % 256).to_string());
                string_data.push_str(" ");
                string_data.push_str(&((j[2] * 255.0) as i64 % 256).to_string());
                string_data.push_str(" ");
            }
            string_data.push_str("\n");
        }

        let mut fout = File::create(path).unwrap();
        fout.write_all(string_data.as_bytes())?;

        return Ok(());
    }
}
