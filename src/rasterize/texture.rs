use std::convert::TryInto;

use serde::{Deserialize, Serialize};

use crate::glm;
use crate::image::Image;

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureRGBAFloat {
    /// id that matches Image id from which the texture is made from
    id: usize,

    width: usize,
    height: usize,
    pixels: Vec<glm::Vec4>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    gl_tex: gl::types::GLuint,
}

impl TextureRGBAFloat {
    pub fn new_empty(width: usize, height: usize) -> Self {
        let gl_tex = Self::gen_gl_texture();
        let pixels = Vec::new();
        let res = Self {
            id: rand::random(),
            width,
            height,
            pixels,
            gl_tex,
        };

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, gl_tex);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA32F.try_into().unwrap(),
                res.width.try_into().unwrap(),
                res.height.try_into().unwrap(),
                0,
                gl::RGBA,
                gl::FLOAT,
                std::ptr::null(),
            )
        }

        res
    }

    pub fn from_pixels(width: usize, height: usize, pixels: Vec<glm::Vec4>) -> Self {
        let gl_tex = Self::gen_gl_texture();
        let res = Self {
            id: rand::random(),
            width,
            height,
            pixels,
            gl_tex,
        };

        assert_eq!(res.pixels.len(), res.width * res.height);

        res.new_texture_to_gl();

        res
    }

    pub fn from_image(tex: &Image) -> Self {
        let mut res = Self::from_pixels(
            tex.width(),
            tex.height(),
            tex.get_pixels()
                .iter()
                .map(|pixel| glm::vec4(pixel[0] as f32, pixel[1] as f32, pixel[2] as f32, 1.0))
                .collect(),
        );
        res.id = tex.get_id();
        res
    }

    pub fn update_from_image(&mut self, tex: &Image) {
        // If the ids are the same, the pixels are also the same so
        // don't do anything
        if self.id == tex.get_id() {
            return;
        }

        *self = Self::from_image(tex);
    }

    /// # Safety
    ///
    /// There is no way to generate [`TextureRGBAFloat`] without
    /// automatically sending the texture to the GPU except during
    /// deserialization so there is no need to call this function
    /// except immediately after deserialization once.
    pub unsafe fn send_to_gpu(&mut self) {
        let gl_tex = Self::gen_gl_texture();
        self.gl_tex = gl_tex;
        assert_eq!(self.pixels.len(), self.width * self.height);

        self.new_texture_to_gl();
    }

    pub fn activate(&mut self, texture_target: u8) {
        let target = match texture_target {
            0 => gl::TEXTURE0,
            1 => gl::TEXTURE1,
            2 => gl::TEXTURE2,
            3 => gl::TEXTURE3,
            4 => gl::TEXTURE4,
            5 => gl::TEXTURE5,
            6 => gl::TEXTURE6,
            7 => gl::TEXTURE7,
            8 => gl::TEXTURE8,
            9 => gl::TEXTURE9,
            10 => gl::TEXTURE10,
            11 => gl::TEXTURE11,
            12 => gl::TEXTURE12,
            13 => gl::TEXTURE13,
            14 => gl::TEXTURE14,
            15 => gl::TEXTURE15,
            16 => gl::TEXTURE16,
            17 => gl::TEXTURE17,
            18 => gl::TEXTURE18,
            19 => gl::TEXTURE19,
            20 => gl::TEXTURE20,
            21 => gl::TEXTURE21,
            22 => gl::TEXTURE22,
            23 => gl::TEXTURE23,
            24 => gl::TEXTURE24,
            25 => gl::TEXTURE25,
            26 => gl::TEXTURE26,
            27 => gl::TEXTURE27,
            28 => gl::TEXTURE28,
            29 => gl::TEXTURE29,
            30 => gl::TEXTURE30,
            31 => gl::TEXTURE31,
            _ => panic!("Texture target not possible, gl support [0, 32)"),
        };
        unsafe {
            gl::ActiveTexture(target);
            gl::BindTexture(gl::TEXTURE_2D, self.gl_tex);
        }
    }

    fn new_texture_to_gl(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.gl_tex);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA32F.try_into().unwrap(),
                self.width.try_into().unwrap(),
                self.height.try_into().unwrap(),
                0,
                gl::RGBA,
                gl::FLOAT,
                self.pixels.as_ptr() as *const gl::types::GLvoid,
            )
        }
    }

    fn gen_gl_texture() -> gl::types::GLuint {
        let mut gl_tex = 0;
        unsafe {
            gl::GenTextures(1, &mut gl_tex);
        }
        assert_ne!(gl_tex, 0);

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, gl_tex);

            // wrapping method
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE.try_into().unwrap(),
            );

            // filter method
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR.try_into().unwrap(),
            );
        }

        gl_tex
    }

    pub fn get_gl_tex(&self) -> gl::types::GLuint {
        self.gl_tex
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_pixels(&self) -> &Vec<glm::Vec4> {
        &self.pixels
    }

    pub fn set_pixel(&mut self, i: usize, j: usize, data: glm::Vec4) {
        self.id = rand::random();
        self.pixels[j * self.width + i] = data;
    }

    pub fn get_pixel(&self, i: usize, j: usize) -> &glm::Vec4 {
        &self.pixels[j * self.width + i]
    }

    /// Get the pixel from the specified UV coordinates
    ///
    /// Wrapping mode is set to repeat. TODO: need to make wrapping
    /// mode user definable
    ///
    /// UV bottom left is (0.0, 0.0) and top right is (1.0, 1.0), same
    /// as OpenGL
    pub fn get_pixel_uv(&self, uv: &glm::DVec2) -> &glm::Vec4 {
        let uv = glm::vec2(uv[0] % 1.0, uv[1] % 1.0);

        self.get_pixel(
            (uv[0] * self.width as f64) as _,
            self.height - (uv[1] * self.height as f64) as usize - 1,
        )
    }
}

impl Drop for TextureRGBAFloat {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.gl_tex);
        }
    }
}
