pub mod bsdf;
pub mod bsdfs;
pub mod camera;
pub mod intersectable;
pub mod ray;
pub mod shader_list;
pub mod shaders;
pub mod traversal_info;

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;

use enumflags2::BitFlags;
use lazy_static::lazy_static;
use rayon::prelude::*;

use crate::glm;
use crate::image::Image;
use crate::path_trace::bsdf::SamplingTypes;
use crate::path_trace::camera::Camera;
use crate::path_trace::intersectable::IntersectInfo;
use crate::path_trace::intersectable::Intersectable;
use crate::path_trace::ray::Ray;
use crate::progress::Progress;
use crate::scene::Scene;

use self::shader_list::Shader;
use self::shader_list::ShaderList;
use self::traversal_info::SingleRayInfo;
use self::traversal_info::TraversalInfo;

lazy_static! {
    static ref DEFAULT_SHADER: self::shaders::Lambert = self::shaders::Lambert::new(
        self::bsdfs::lambert::Lambert::new(glm::vec4(0.0, 0.0, 0.0, 1.0))
    );
}

pub struct RayTraceParams {
    width: usize,
    height: usize,
    trace_max_depth: usize,
    samples_per_pixel: usize,
}

impl RayTraceParams {
    pub fn new(
        width: usize,
        height: usize,
        trace_max_depth: usize,
        samples_per_pixel: usize,
    ) -> Self {
        Self {
            width,
            height,
            trace_max_depth,
            samples_per_pixel,
        }
    }

    /// Get ray trace params's width.
    pub fn get_width(&self) -> usize {
        self.width
    }

    /// Get ray trace params's height.
    pub fn get_height(&self) -> usize {
        self.height
    }

    /// Get ray trace params's trace_max_depth.
    pub fn get_trace_max_depth(&self) -> usize {
        self.trace_max_depth
    }

    /// Get ray trace params's samples_per_pixel.
    pub fn get_samples_per_pixel(&self) -> usize {
        self.samples_per_pixel
    }
}

fn ray_trace_scene(
    ray_trace_params: RayTraceParams,
    scene: Arc<RwLock<Scene>>,
    shader_list: Arc<RwLock<ShaderList>>,
    camera: Arc<RwLock<Camera>>,
    rendered_image: Arc<RwLock<Image>>,
    progress: Arc<RwLock<Progress>>,
    stop_render: Arc<RwLock<bool>>,
) {
    let mut image = Image::new(ray_trace_params.get_width(), ray_trace_params.get_height());
    progress.write().unwrap().reset();

    let camera = camera.read().unwrap();

    let progress_previous_update = Arc::new(RwLock::new(Instant::now()));
    let total_number_of_samples = ray_trace_params.get_samples_per_pixel()
        * ray_trace_params.get_width()
        * ray_trace_params.get_height();

    // ray trace
    for processed_samples in 0..ray_trace_params.get_samples_per_pixel() {
        if *stop_render.read().unwrap() {
            progress.write().unwrap().stop_progress();
            return;
        }

        let processed_pixels = Arc::new(AtomicUsize::new(0));

        scene.write().unwrap().apply_model_matrices();

        let scene = scene.read().unwrap();
        let shader_list = shader_list.read().unwrap();
        let image_width = image.width();
        image
            .get_pixels_mut()
            .par_iter_mut()
            .chunks(image_width)
            .enumerate()
            .for_each(|(j, mut row)| {
                row.par_iter_mut().enumerate().for_each(|(i, pixel)| {
                    let processed_pixels = processed_pixels.fetch_add(1, Ordering::SeqCst);

                    if progress_previous_update
                        .read()
                        .unwrap()
                        .elapsed()
                        .as_secs_f64()
                        > 0.03
                    {
                        let calculated_progress = (processed_samples
                            * ray_trace_params.get_width()
                            * ray_trace_params.get_height()
                            + processed_pixels)
                            as f64
                            / total_number_of_samples as f64;

                        progress.write().unwrap().set_progress(calculated_progress);

                        *progress_previous_update.write().unwrap() = Instant::now();
                    }

                    let j = ray_trace_params.get_height() - j - 1;

                    // use opengl coords, (0.0, 0.0) is center; (1.0, 1.0) is
                    // top right; (-1.0, -1.0) is bottom left
                    let u = (((i as f64 + rand::random::<f64>())
                        / (ray_trace_params.get_width() - 1) as f64)
                        - 0.5)
                        * 2.0;
                    let v = (((j as f64 + rand::random::<f64>())
                        / (ray_trace_params.get_height() - 1) as f64)
                        - 0.5)
                        * 2.0;

                    let ray = camera.get_ray(u, v);

                    let (color, _traversal_info) = trace_ray(
                        &ray,
                        &camera,
                        &scene,
                        ray_trace_params.get_trace_max_depth(),
                        &shader_list,
                    );

                    **pixel += color;
                });
            });

        {
            let mut rendered_image = rendered_image.write().unwrap();
            *rendered_image = image.clone();
            rendered_image
                .get_pixels_mut()
                .par_iter_mut()
                .for_each(|pixel| {
                    *pixel /= (processed_samples + 1) as f64;
                });
        }

        {
            let mut progress = progress.write().unwrap();
            progress.set_progress(
                (processed_samples + 1) as f64 / ray_trace_params.get_samples_per_pixel() as f64,
            );
        }
    }

    scene.write().unwrap().unapply_model_matrices();
}

pub enum RayTraceMessage {
    StartRender(RayTraceParams),
    StopRender,
    KillThread,
}

fn ray_trace_stop_render(
    stop_render: Arc<RwLock<bool>>,
    render_thread_handle: Option<JoinHandle<()>>,
) -> Option<JoinHandle<()>> {
    *stop_render.write().unwrap() = true;
    let render_thread_handle = render_thread_handle.and_then(|join_handle| {
        join_handle.join().unwrap();
        None
    });
    *stop_render.write().unwrap() = false;
    render_thread_handle
}

pub fn ray_trace_main(
    scene: Arc<RwLock<Scene>>,
    shader_list: Arc<RwLock<ShaderList>>,
    camera: Arc<RwLock<Camera>>,
    rendered_image: Arc<RwLock<Image>>,
    progress: Arc<RwLock<Progress>>,
    message_receiver: Receiver<RayTraceMessage>,
) {
    let stop_render = Arc::new(RwLock::new(false));
    let mut render_thread_handle: Option<JoinHandle<()>> = None;

    loop {
        let message = message_receiver.recv().unwrap();
        match message {
            RayTraceMessage::StartRender(params) => {
                // stop any previously running ray traces
                ray_trace_stop_render(stop_render.clone(), render_thread_handle);

                let scene = scene.clone();
                let shader_list = shader_list.clone();
                let camera = camera.clone();
                let rendered_image = rendered_image.clone();
                let progress = progress.clone();
                let stop_render = stop_render.clone();
                render_thread_handle = Some(thread::spawn(move || {
                    ray_trace_scene(
                        params,
                        scene,
                        shader_list,
                        camera,
                        rendered_image,
                        progress,
                        stop_render,
                    );
                }));
            }
            RayTraceMessage::StopRender => {
                render_thread_handle =
                    ray_trace_stop_render(stop_render.clone(), render_thread_handle);
            }
            RayTraceMessage::KillThread => {
                break;
            }
        }
    }
}

pub enum ShadeHitData {
    Both(ShadeHitDataBoth),
    ScatterOnly(ShadeHitDataScatterOnly),
    EmissionOnly(ShadeHitDataEmissionOnly),
    None,
}

/// Data that is returned during the `shade_hit()` calculation when
/// light is scattered and emission takes place
#[derive(Debug, Clone, PartialEq)]
pub struct ShadeHitDataBoth {
    /// color information that should be propagated forward
    color: glm::DVec3,
    /// color of light produced with intensity of the light encoded
    emission_color: glm::DVec3,
    /// the next ray to continue the ray tracing, calculated from the
    /// `BSDF`
    next_ray: Ray,
    /// type of sampling performed to generate the next ray by the
    /// `BSDF`
    sampling_type: SamplingTypes,
}

impl ShadeHitDataBoth {
    pub fn new(
        color: glm::DVec3,
        emission_color: glm::DVec3,
        next_ray: Ray,
        sampling_type: SamplingTypes,
    ) -> Self {
        Self {
            color,
            emission_color,
            next_ray,
            sampling_type,
        }
    }

    pub fn get_color(&self) -> &glm::DVec3 {
        &self.color
    }

    pub fn get_emission_color(&self) -> &glm::DVec3 {
        &self.emission_color
    }

    pub fn get_next_ray(&self) -> &Ray {
        &self.next_ray
    }

    pub fn get_sampling_type(&self) -> SamplingTypes {
        self.sampling_type
    }
}

/// Data that is returned during the `shade_hit()` calculation when
/// light is scattered only
#[derive(Debug, Clone, PartialEq)]
pub struct ShadeHitDataScatterOnly {
    /// color information that should be propagated forward
    color: glm::DVec3,
    /// the next ray to continue the ray tracing, calculated from the
    /// `BSDF`
    next_ray: Ray,
    /// type of sampling performed to generate the next ray by the
    /// `BSDF`
    sampling_type: SamplingTypes,
}

impl ShadeHitDataScatterOnly {
    pub fn new(color: glm::DVec3, next_ray: Ray, sampling_type: SamplingTypes) -> Self {
        Self {
            color,
            next_ray,
            sampling_type,
        }
    }

    pub fn get_color(&self) -> &glm::DVec3 {
        &self.color
    }

    pub fn get_next_ray(&self) -> &Ray {
        &self.next_ray
    }

    pub fn get_sampling_type(&self) -> SamplingTypes {
        self.sampling_type
    }
}

/// Data that is returned during the `shade_hit()` calculation when
/// emission takes place only
#[derive(Debug, Clone, PartialEq)]
pub struct ShadeHitDataEmissionOnly {
    /// color of light produced with intensity of the light encoded
    emission_color: glm::DVec3,
}

impl ShadeHitDataEmissionOnly {
    pub fn new(emission_color: glm::DVec3) -> Self {
        Self { emission_color }
    }

    pub fn get_emission_color(&self) -> &glm::DVec3 {
        &self.emission_color
    }
}

fn shade_environment(ray: &Ray, camera: &Camera) -> glm::DVec3 {
    let color_1 = glm::vec3(0.8, 0.8, 0.8);
    let color_2 = glm::vec3(0.2, 0.2, 0.8);

    let camera_origin_y = camera.get_origin()[1];
    let camera_vertical_range = camera.get_vertical()[1];
    let y_val = (camera_origin_y + ray.get_direction()[1]) / camera_vertical_range;
    let y_val = (y_val + 1.0) / 2.0;
    let y_val = y_val.clamp(0.0, 1.0);

    glm::lerp(&color_1, &color_2, y_val)
}

/// Shade the point of intersection when the ray hits an object
fn shade_hit(ray: &Ray, intersect_info: &IntersectInfo, shader_list: &ShaderList) -> ShadeHitData {
    // TODO: currently using a default shader only if the shader has
    // been deleted but there is no way to inform this to the user as
    // of now. Need to figure out a way to let the user know that the
    // object doesn't have a shader valid assigned.
    let shader = match shader_list.get_shader(intersect_info.get_shader_id().unwrap()) {
        Some(shader) => shader.get_bsdf(),
        None => {
            // use a default shader when shader is no longer available in the shader_list
            DEFAULT_SHADER.get_bsdf()
        }
    };

    // wo: outgoing ray direction
    //
    // Outgoing ray direction must be the inverse of the current ray since
    // the current ray are travelling from camera into the scene and the
    // BSDF need not care about that. It must receive only the outgoing
    // direction.
    let wo = -ray.get_direction();

    // wi: incoming way direction
    let op_sample_data = shader.sample(&wo, intersect_info, BitFlags::all());

    if let Some(sample_data) = op_sample_data {
        let wi = sample_data.get_wi().normalize();
        let sampling_type = sample_data.get_sampling_type();

        let color = shader.eval(&wi, &wo, intersect_info);

        // BSDF returns the incoming ray direction at the point of
        // intersection but for the next ray that is shot in the opposite
        // direction (into the scene), thus need to take the inverse of
        // `wi`.
        let next_ray_dir = -wi;

        let emission = shader.emission(intersect_info);
        if let Some(emission) = emission {
            ShadeHitData::Both(ShadeHitDataBoth::new(
                color,
                emission,
                Ray::new(*intersect_info.get_point(), next_ray_dir),
                sampling_type,
            ))
        } else {
            ShadeHitData::ScatterOnly(ShadeHitDataScatterOnly::new(
                color,
                Ray::new(*intersect_info.get_point(), next_ray_dir),
                sampling_type,
            ))
        }
    } else {
        let emission = shader.emission(intersect_info);
        if let Some(emission) = emission {
            ShadeHitData::EmissionOnly(ShadeHitDataEmissionOnly::new(emission))
        } else {
            ShadeHitData::None
        }
    }
}

// x: current point
// x_prime: previous point
// x_prime_prime: previous's previous point
// g: geometry term, 1/(r^2) where r is distance of x_prime to x
// e: intensity of emitted light by x_prime reaching x
// i: intensity of light from x_prime to x
// p: intensity of light scattered from x_prime_prime to x by a patch on surface at x_prime
/// Traces the given ray into the scene and returns the
/// colour/intensity of light propagated by the given along with the
/// path traced till that point
pub fn trace_ray(
    ray: &Ray,
    camera: &Camera,
    scene: &Scene,
    depth: usize,
    shader_list: &ShaderList,
) -> (glm::DVec3, TraversalInfo) {
    if depth == 0 {
        return (glm::zero(), TraversalInfo::new());
    }
    if let Some(info) = scene.hit(ray, 0.01, 1000.0) {
        match shade_hit(ray, &info, shader_list) {
            ShadeHitData::Both(ShadeHitDataBoth {
                color,
                emission_color,
                next_ray,
                sampling_type: _,
            }) => {
                let (traced_color, mut traversal_info) = trace_ray(&next_ray, camera, scene, depth - 1, shader_list);
                let val = emission_color
                    + glm::vec3(
                        color[0] * traced_color[0],
                        color[1] * traced_color[1],
                        color[2] * traced_color[2],
                    );
                traversal_info.add_ray(SingleRayInfo::new(*ray, Some(*info.get_point()), val, Some(info.get_normal().unwrap())));
                (val, traversal_info)
            }
            ShadeHitData::ScatterOnly(ShadeHitDataScatterOnly {
                color,
                next_ray,
                sampling_type: _,
            }) => {
                let (traced_color, mut traversal_info) = trace_ray(&next_ray, camera, scene, depth - 1, shader_list);
                let val = glm::vec3(
                    color[0] * traced_color[0],
                    color[1] * traced_color[1],
                    color[2] * traced_color[2],
                );
                traversal_info.add_ray(SingleRayInfo::new(*ray, Some(*info.get_point()), val, Some(info.get_normal().unwrap())));
                (val, traversal_info)
            }
            ShadeHitData::EmissionOnly(ShadeHitDataEmissionOnly { emission_color }) => {
                let val = emission_color;
                let mut traversal_info = TraversalInfo::new();
                traversal_info.add_ray(SingleRayInfo::new(*ray, Some(*info.get_point()), val, Some(info.get_normal().unwrap())));
                (val, traversal_info)
            }
            ShadeHitData::None => unreachable!(
                "No shade_hit() should return ShadeHitData::None, it must either scatter or emit or both"
            ),
        }
    } else {
        let mut traversal_info = TraversalInfo::new();
        let color = shade_environment(ray, camera);
        traversal_info.add_ray(SingleRayInfo::new(*ray, None, color, None));
        (color, traversal_info)
    }
}
