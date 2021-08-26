use rt::camera::Camera;
use rt::image::{Image, PPM};
use rt::math::Scalar;
use rt::scene::Scene;
use rt::sphere::Sphere;

use rt::trace_ray;

use nalgebra_glm as glm;
extern crate lazy_static;
use crossbeam::thread;
use lazy_static::lazy_static;

lazy_static! {
    static ref SCENE: Scene = {
        let mut scene = Scene::new();
        scene.add_object(Box::new(Sphere::new(glm::vec3(0.0, 0.0, -2.0), 1.5)));
        scene.add_object(Box::new(Sphere::new(glm::vec3(0.0, 1.0, -2.0), 1.5)));
        scene.add_object(Box::new(Sphere::new(glm::vec3(0.0, -1.0, -2.0), 1.5)));
        scene.add_object(Box::new(Sphere::new(glm::vec3(1.0, 0.0, -2.0), 1.5)));
        scene.add_object(Box::new(Sphere::new(glm::vec3(-1.0, 0.0, -2.0), 1.5)));
        scene
    };
}

fn main() {
    let width = 1000;
    let height = 1000;
    let mut image = Image::new(width, height);

    let viewport_height = 2.0;
    let aspect_ratio = width as f64 / height as f64;
    let focal_length = 1.0;
    let origin = glm::vec3(0.0, 0.0, 0.0);
    let camera = Camera::new(viewport_height, aspect_ratio, focal_length, origin);
    let camera = &camera;

    {
        let num_threads = 12;
        let mut slabs = image.get_slabs(num_threads);

        println!("slabs: {:?}", slabs);

        thread::scope(|s| {
            let mut handles = Vec::new();

            for slab in &mut slabs {
                let handle = s.spawn(move |_| {
                    let mut pixels = Vec::new();
                    for i in 0..slab.width {
                        let mut pixels_inner = Vec::new();
                        for j in 0..slab.height {
                            let j = j + slab.y_start;
                            let j = height - j;
                            let i = i + slab.x_start;

                            // use opengl coords, (0.0, 0.0) is center; (1.0, 1.0) is
                            // top right; (-1.0, -1.0) is bottom left
                            let u = ((i as Scalar / (width - 1) as Scalar) - 0.5) * 2.0;
                            let v = ((j as Scalar / (height - 1) as Scalar) - 0.5) * 2.0;

                            let ray = camera.get_ray(u, v);

                            let pixel = trace_ray(&ray, camera, &SCENE, 2000);
                            pixels_inner.push(pixel);
                        }
                        pixels.push(pixels_inner);
                    }

                    slab.set_pixels(pixels);
                });

                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        })
        .unwrap();

        for slab in slabs {
            for i in 0..slab.width {
                for j in 0..slab.height {
                    let pixel = slab.get_pixels()[i][j];
                    let j = j + slab.y_start;
                    let i = i + slab.x_start;

                    image.set_pixel(j, i, pixel);
                }
            }
        }
    }

    // for (j, row) in image.get_pixels_mut().iter_mut().enumerate() {
    //     for (i, pixel) in row.iter_mut().enumerate() {
    //         let j = height - j - 1;

    //         // use opengl coords, (0.0, 0.0) is center; (1.0, 1.0) is
    //         // top right; (-1.0, -1.0) is bottom left
    //         let u = ((i as Scalar / (width - 1) as Scalar) - 0.5) * 2.0;
    //         let v = ((j as Scalar / (height - 1) as Scalar) - 0.5) * 2.0;

    //         let ray = camera.get_ray(u, v);

    //         *pixel = trace_ray(&ray, &camera, &SCENE, 2);
    //     }
    // }

    let ppm = PPM::new(&image);
    ppm.write_to_file("image.ppm").unwrap();
}
