use nalgebra::RealField;

use crate::glm;
use crate::image::Image;

use super::spectrum::TSpectrum;

#[derive(Debug, Clone)]
pub struct TSpectralImage<T> {
    /// If the id has changed, the pixel data might have also
    /// changed. Every time the image is borrowed mutably, the id is
    /// update.
    id: usize,

    /// pixels of the image stored as spectrum stored from top left
    /// row wise
    pixels: Vec<TSpectrum<T>>,

    width: usize,
    height: usize,
}

impl<T: std::clone::Clone> TSpectralImage<T> {
    pub fn new(width: usize, height: usize) -> Self {
        let mut pixels = Vec::with_capacity(width * height);
        pixels.resize(width * height, TSpectrum::new_empty());

        Self::from_pixels(width, height, pixels)
    }
}

impl<T> TSpectralImage<T> {
    pub fn from_pixels(width: usize, height: usize, pixels: Vec<TSpectrum<T>>) -> Self {
        Self {
            id: rand::random(),
            pixels,
            width,
            height,
        }
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_pixels(&self) -> &[TSpectrum<T>] {
        self.pixels.as_ref()
    }

    pub fn get_pixels_mut(&mut self) -> &mut [TSpectrum<T>] {
        self.id = rand::random();
        self.pixels.as_mut()
    }
}

impl<T: RealField + simba::scalar::SubsetOf<f64>> TSpectralImage<T> {
    /// Convert spectral image to sRGB [`Image`]
    pub fn to_image_srgb(&self) -> Image {
        Image::from_pixels(
            self.get_width(),
            self.get_height(),
            self.get_pixels()
                .iter()
                .map(|pixel| glm::convert(pixel.to_srgb()))
                .collect(),
        )
    }
}

impl<T: RealField + simba::scalar::SubsetOf<f64> + simba::scalar::SubsetOf<f32>> TSpectralImage<T> {
    /// Convert spectral image to linear RGB [`Image`]
    pub fn to_image_rgb(&self) -> Image {
        Image::from_pixels(
            self.get_width(),
            self.get_height(),
            self.get_pixels()
                .iter()
                .map(|pixel| glm::convert(pixel.to_rgb()))
                .collect(),
        )
    }
}

pub type DSpectralImage = TSpectralImage<f64>;
