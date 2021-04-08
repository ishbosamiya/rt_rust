use rt::image::{Image, PPM};
use rt::math::Scalar;

use nalgebra_glm as glm;

fn main() {
    let width = 1280;
    let height = 720;
    let mut image = Image::new(width, height);

    for (j, row) in image.get_pixels_mut().iter_mut().enumerate() {
        for (i, pixel) in row.iter_mut().enumerate() {
            let j = height - j - 1;

            // use opengl coords, (0.0, 0.0) is center; (1.0, 1.0) is
            // top right; (-1.0, -1.0) is bottom left
            let u = ((i as Scalar / (width - 1) as Scalar) - 0.5) * 2.0;
            let v = ((j as Scalar / (height - 1) as Scalar) - 0.5) * 2.0;

            *pixel = glm::vec3(u, v, 0.0);
        }
    }

    let ppm = PPM::new(&image);
    ppm.write_to_file("image.ppm").unwrap();
}
