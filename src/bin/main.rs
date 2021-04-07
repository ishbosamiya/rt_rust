use hp_rt::image::{Image, PPM};

use nalgebra_glm as glm;

fn main() {
    let width = 1280;
    let height = 720;
    let mut image = Image::new(width, height);

    for (_j, row) in image.get_pixels_mut().iter_mut().enumerate() {
        for (_i, pixel) in row.iter_mut().enumerate() {
            let _j = height - _j - 1;

            *pixel = glm::vec3(0.3, 0.3, 0.3);
        }
    }

    let ppm = PPM::new(&image);
    ppm.write_to_file("image.ppm").unwrap();
}
