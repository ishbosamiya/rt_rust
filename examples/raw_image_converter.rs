/// Raw image converter
///
/// A simple converter to convert from the custom format part of `rt`
/// to standard formats while maybe converting linear to srgb
use std::convert::TryInto;

use rt::image::Image;

fn main() {
    let matches = clap::App::new("Raw Image Converter")
        .version("0.1.0")
        .about("Simple converter to convert from custom format to standard formats")
        .arg(
            clap::Arg::with_name("input")
                .short("i")
                .help("input raw image locations")
                .long_help("if multiple file locations given, combines them linearly before exporting the image")
                .required(true)
                .multiple(true)
                .takes_value(true),
        )
        .arg(clap::Arg::with_name("linear_to_srgb").help("convert linear to srgb"))
        .arg(
            clap::Arg::with_name("output")
                .short("o")
                .help("output file location")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let input_file_location = matches.values_of("input").unwrap();
    let output_file_location = matches.value_of("output").unwrap();
    let linear_to_srgb = matches.value_of("linear_to_srgb").is_some();

    println!("input file location: {:?}", input_file_location);
    println!("output file location: {}", output_file_location);
    println!("linear_to_srgb: {}", linear_to_srgb);

    let images = load_images(&input_file_location.collect::<Vec<_>>());

    let image = combine_images(&images).unwrap();

    let image = image::ImageBuffer::from_fn(
        image.width().try_into().unwrap(),
        image.height().try_into().unwrap(),
        |i, j| {
            let pixel = image.get_pixel(i.try_into().unwrap(), j.try_into().unwrap());
            let pixel = [pixel[0] as f32, pixel[1] as f32, pixel[2] as f32];

            let pixel = if linear_to_srgb {
                [
                    egui::color::gamma_from_linear(pixel[0]),
                    egui::color::gamma_from_linear(pixel[1]),
                    egui::color::gamma_from_linear(pixel[2]),
                ]
            } else {
                pixel
            };

            let pixel = [
                (pixel[0] * 255.0).round(),
                (pixel[1] * 255.0).round(),
                (pixel[2] * 255.0).round(),
                255.0,
            ];

            image::Rgba([
                pixel[0] as u8,
                pixel[1] as u8,
                pixel[2] as u8,
                pixel[3] as u8,
            ])
        },
    );

    image.save(output_file_location).unwrap();
}

fn load_images<P>(paths: &[P]) -> Vec<Image>
where
    P: AsRef<std::path::Path>,
{
    paths
        .iter()
        .map(|path| {
            let file = std::fs::read(path).unwrap();
            serde_json::from_slice(&file).unwrap()
        })
        .collect()
}

fn combine_images(images: &[Image]) -> Option<Image> {
    if images.is_empty() {
        None
    } else {
        let mut image = images.iter().try_fold(
            Image::new(images[0].width(), images[0].height()),
            |acc, image| {
                if acc.width() != image.width() || acc.height() != image.height() {
                    None
                } else {
                    Some(Image::from_pixels(
                        image.width(),
                        image.height(),
                        acc.get_pixels()
                            .iter()
                            .zip(image.get_pixels().iter())
                            .map(|(p1, p2)| p1 + p2)
                            .collect(),
                    ))
                }
            },
        )?;
        image.get_pixels_mut().iter_mut().for_each(|pixel| {
            *pixel /= images.len() as f64;
        });
        Some(image)
    }
}
