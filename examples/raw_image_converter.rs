/// Raw image converter
///
/// A simple converter to convert from the custom format part of `rt`
/// to standard formats while maybe converting linear to srgb
use std::{convert::TryInto, path::PathBuf};

use rt::image::Image;

use clap::{value_t, values_t};

fn main() {
    let matches = clap::App::new("Raw Image Converter")
        .version("0.1.0")
        .about("Simple converter to convert from custom format to standard formats")
        .arg(clap::Arg::with_name("input-directory").long("input-directory").short("d").required_unless("input").conflicts_with_all(&["input", "output"]).takes_value(true).number_of_values(1).requires_all(&["output-type"]))
        .arg(clap::Arg::with_name("output-type").long("output-type").takes_value(true).number_of_values(1))
        .arg(
            clap::Arg::with_name("input")
                .short("i")
                .help("input raw image locations")
                .long_help("if multiple file locations given, combines them linearly before exporting the image")
                .required_unless("input-directory")
                .multiple(true)
                .takes_value(true),
        )
        .arg(clap::Arg::with_name("linear-to-srgb").long("--linear-to-srgb").help("convert linear to srgb"))
        .arg(
            clap::Arg::with_name("output")
                .short("o")
                .help("output file location")
                .required_unless("input-directory")
                .takes_value(true),
        )
        .get_matches();

    let linear_to_srgb = matches.is_present("linear-to-srgb");
    dbg!(linear_to_srgb);

    if let Some(input_dir) = clap::value_t!(matches, "input-directory", PathBuf).ok() {
        dbg!(&input_dir);
        assert!(input_dir.is_dir());

        let output_type = clap::value_t!(matches, "output-type", String).unwrap();
        dbg!(&output_type);
        if output_type != "png" && output_type != "jpg" && output_type != "tiff" {
            panic!("Output file type is not support");
        }

        std::fs::read_dir(input_dir).unwrap().for_each(|file| {
            let file = file.unwrap().path();
            if let Some(file_type) = file.extension() {
                if file_type == "image" {
                    let images = load_images(&[&file]);
                    let image = combine_images(&images).unwrap();
                    let mut output_file_location = file.clone();
                    output_file_location.set_extension(&output_type);
                    save_image(&image, linear_to_srgb, &output_file_location);
                    println!(
                        "Saved {} as {}",
                        file.to_str().unwrap(),
                        output_file_location.to_str().unwrap()
                    );
                }
            }
        });
    } else {
        let input_file_location: Vec<PathBuf> = clap::values_t!(matches, "input", PathBuf).unwrap();
        let output_file_location: PathBuf = clap::value_t!(matches, "output", PathBuf).unwrap();

        dbg!(&input_file_location);
        dbg!(&output_file_location);

        let images = load_images(&input_file_location);
        let image = combine_images(&images).unwrap();
        save_image(&image, linear_to_srgb, output_file_location);
    }
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

fn save_image<P>(image: &Image, linear_to_srgb: bool, output_path: P)
where
    P: AsRef<std::path::Path>,
{
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

    image.save(output_path).unwrap();
}
