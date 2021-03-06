pub mod bsdf;
pub mod bsdfs;
pub mod environment;
pub mod intersectable;
pub mod medium;
pub mod ray;
pub mod shader_list;
pub mod shaders;
pub mod texture_list;
pub mod traversal_info;
pub mod viewport_renderer;

use enumflags2::BitFlags;
use lazy_static::lazy_static;
use quick_renderer::camera::Camera;
use rayon::prelude::*;

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::Receiver,
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

use crate::{
    camera::CameraExtension,
    glm,
    image::Image,
    path_trace::{
        bsdf::SamplingTypes,
        intersectable::{IntersectInfo, Intersectable},
        ray::Ray,
    },
    progress::Progress,
    scene::Scene,
    util,
};

use self::{
    environment::{Environment, EnvironmentShadingData},
    medium::Mediums,
    shader_list::{Shader, ShaderList},
    texture_list::TextureList,
    traversal_info::{SingleRayInfo, TraversalInfo},
};

lazy_static! {
    static ref DEFAULT_SHADER: self::shaders::Lambert =
        self::shaders::Lambert::new(self::bsdfs::lambert::Lambert::new(glm::vec3(0.0, 0.0, 0.0)));
}

#[derive(Debug, Clone)]
pub struct RayTraceParams {
    /// width of the ray trace render
    width: usize,
    /// height of the ray trace render
    height: usize,
    /// max depth the trace can traverse
    trace_max_depth: usize,
    /// number of samples (rays traced) per pixel
    samples_per_pixel: usize,
    /// camera used for ray tracing
    ///
    /// Side note: it might at first seem like a good idea to have the
    /// Camera wrapped in a Arc and RwLock but this has downsides that
    /// that the camera must either be read locked for a long duration
    /// making edits to the camera not easy if not impossible or read
    /// locks must be done often enough that that becomes an expensive
    /// operation. It is better store a clone of the camera since
    /// changes to the camera should anyway not be propagated while
    /// tracing the scene.
    camera: Camera,
    /// image to which the render (can be progressive) is updated
    rendered_image: Arc<RwLock<Image>>,
}

impl RayTraceParams {
    pub fn new(
        width: usize,
        height: usize,
        trace_max_depth: usize,
        samples_per_pixel: usize,
        camera: Camera,
        rendered_image: Arc<RwLock<Image>>,
    ) -> Self {
        Self {
            width,
            height,
            trace_max_depth,
            samples_per_pixel,
            camera,
            rendered_image,
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

    /// Get a reference to ray trace params's camera.
    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    /// Get ray trace params's rendered image.
    pub fn get_rendered_image(&self) -> Arc<RwLock<Image>> {
        self.rendered_image.clone()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ray_trace_scene(
    ray_trace_params: RayTraceParams,
    scene: Arc<RwLock<Scene>>,
    shader_list: Arc<RwLock<ShaderList>>,
    texture_list: Arc<RwLock<TextureList>>,
    environment: Arc<RwLock<Environment>>,
    progress: Arc<RwLock<Progress>>,
    stop_render: Arc<RwLock<bool>>,
    stop_render_immediate: Arc<RwLock<bool>>,
) {
    let mut image = Image::new(ray_trace_params.get_width(), ray_trace_params.get_height());
    progress.write().unwrap().reset();

    let camera = ray_trace_params.get_camera();

    let update_often = Arc::new(RwLock::new(Instant::now()));
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

        scene.write().unwrap().rebuild_bvh_if_needed(0.01);

        let scene = scene.read().unwrap();
        let shader_list = shader_list.read().unwrap();
        let texture_list = texture_list.read().unwrap();
        let environment: &Environment = &environment.read().unwrap();
        let environment = environment.into();
        let image_width = image.width();
        let maybe_exit = image
            .get_pixels_mut()
            .par_iter_mut()
            .chunks(image_width)
            .enumerate()
            .try_for_each(|(j, mut row)| {
                row.par_iter_mut().enumerate().try_for_each(|(i, pixel)| {
                    let processed_pixels = processed_pixels.fetch_add(1, Ordering::SeqCst);

                    {
                        if update_often.read().unwrap().elapsed().as_secs_f64() > 0.03 {
                            // calculate and set progress
                            {
                                let calculated_progress = (processed_samples
                                    * ray_trace_params.get_width()
                                    * ray_trace_params.get_height()
                                    + processed_pixels)
                                    as f64
                                    / total_number_of_samples as f64;

                                progress.write().unwrap().set_progress(calculated_progress);
                            }

                            // check if render must be stopped immediately
                            if *stop_render_immediate.read().unwrap() {
                                progress.write().unwrap().stop_progress();
                                return None;
                            }

                            *update_often.write().unwrap() = Instant::now();
                        }
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

                    let ray = camera.get_ray(&glm::vec2(u, v)).unwrap();

                    let (color, _traversal_info) = trace_ray(
                        &ray,
                        camera,
                        &scene,
                        ray_trace_params.get_trace_max_depth(),
                        &shader_list,
                        &texture_list,
                        &environment,
                        &mut Mediums::with_air(),
                    );

                    **pixel += color;

                    Some(())
                })?;
                Some(())
            });

        // little bit confusing, but the loops return None if early
        // exit must be done
        if maybe_exit.is_none() {
            return;
        }

        {
            let mut rendered_image = ray_trace_params.rendered_image.write().unwrap();
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
    StartRender(Box<RayTraceParams>),
    FinishSampleAndStopRender,
    StopRenderImmediately,
    KillThread,
    FinishAndKillThread,
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

#[allow(clippy::too_many_arguments)]
pub fn ray_trace_main(
    scene: Arc<RwLock<Scene>>,
    shader_list: Arc<RwLock<ShaderList>>,
    texture_list: Arc<RwLock<TextureList>>,
    environment: Arc<RwLock<Environment>>,
    progress: Arc<RwLock<Progress>>,
    message_receiver: Receiver<RayTraceMessage>,
) {
    let stop_render = Arc::new(RwLock::new(false));
    let stop_render_immediate = Arc::new(RwLock::new(false));
    let mut render_thread_handle: Option<JoinHandle<()>> = None;

    loop {
        let message = message_receiver.recv().unwrap();
        match message {
            RayTraceMessage::StartRender(params) => {
                // stop any previously running ray traces
                ray_trace_stop_render(stop_render_immediate.clone(), render_thread_handle);

                let scene = scene.clone();
                let shader_list = shader_list.clone();
                let texture_list = texture_list.clone();
                let environment = environment.clone();
                let progress = progress.clone();
                let stop_render = stop_render.clone();
                let stop_render_immediate = stop_render_immediate.clone();
                render_thread_handle = Some(thread::spawn(move || {
                    ray_trace_scene(
                        *params,
                        scene,
                        shader_list,
                        texture_list,
                        environment,
                        progress,
                        stop_render,
                        stop_render_immediate,
                    );
                }));
            }
            RayTraceMessage::FinishSampleAndStopRender => {
                render_thread_handle =
                    ray_trace_stop_render(stop_render.clone(), render_thread_handle);
            }
            RayTraceMessage::StopRenderImmediately => {
                render_thread_handle =
                    ray_trace_stop_render(stop_render_immediate.clone(), render_thread_handle);
            }
            RayTraceMessage::KillThread => {
                break;
            }
            RayTraceMessage::FinishAndKillThread => {
                if let Some(handle) = render_thread_handle {
                    handle.join().unwrap();
                }
                break;
            }
        }
    }
}

/// Data returned during shading of the hitpoint
pub type ShadeHitData = (Option<ScatterHitData>, Option<EmissionHitData>);

/// Data returned during scattering of light while shading of the
/// hitpoint
#[derive(Debug, Clone, PartialEq)]
pub struct ScatterHitData {
    /// color information that should be propagated forward
    color: glm::DVec3,
    /// the next ray to continue the ray tracing, calculated from the
    /// `BSDF`
    next_ray: Ray,
    /// type of sampling performed to generate the next ray by the
    /// `BSDF`
    sampling_type: SamplingTypes,
}

impl ScatterHitData {
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

/// Data returned during emission of light while shading of the
/// hitpoint
#[derive(Debug, Clone, PartialEq)]
pub struct EmissionHitData {
    /// color of light produced with intensity of the light encoded
    emission_color: glm::DVec3,
}

impl EmissionHitData {
    pub fn new(emission_color: glm::DVec3) -> Self {
        Self { emission_color }
    }

    pub fn get_emission_color(&self) -> &glm::DVec3 {
        &self.emission_color
    }
}

pub fn direction_to_equirectangular_range(dir: &glm::DVec3, range: &glm::DVec4) -> glm::DVec2 {
    let u = (-dir[2].atan2(dir[0]) - range[1]) / range[0];
    let v = ((dir[1] / dir.norm()).acos() - range[3]) / range[2];

    glm::vec2(u, v)
}

pub fn direction_to_equirectangular(dir: &glm::DVec3) -> glm::DVec2 {
    direction_to_equirectangular_range(
        dir,
        &glm::vec4(
            -std::f64::consts::TAU,
            std::f64::consts::PI,
            -std::f64::consts::PI,
            std::f64::consts::PI,
        ),
    )
}

fn shade_environment(ray: &Ray, environment: &EnvironmentShadingData) -> glm::DVec3 {
    let transformed_direction = util::vec3_apply_model_matrix(
        ray.get_direction(),
        &environment.get_transform().get_matrix(),
    );

    let uv = direction_to_equirectangular(&transformed_direction);
    *environment.get_hdr().get_pixel_uv(&uv) * environment.get_strength()
}

/// Shade the point of intersection when the ray hits an object
fn shade_hit(
    ray: &Ray,
    intersect_info: &IntersectInfo,
    shader_list: &ShaderList,
    texture_list: &TextureList,
    mediums: &mut Mediums,
) -> ShadeHitData {
    // TODO: currently using a default shader only if the shader has
    // been deleted but there is no way to inform this to the user as
    // of now. Need to figure out a way to let the user know that the
    // object doesn't have a shader valid assigned.
    let bsdf = intersect_info
        .get_shader_id()
        .and_then(|shader_id| shader_list.get_shader(shader_id))
        .map_or(DEFAULT_SHADER.get_bsdf(), |shader| shader.get_bsdf());

    // wo: outgoing ray direction
    //
    // Outgoing ray direction must be the inverse of the current ray since
    // the current ray are travelling from camera into the scene and the
    // BSDF need not care about that. It must receive only the outgoing
    // direction.
    let wo = -ray.get_direction();

    let scattering_data = bsdf
        .sample(&wo, mediums, intersect_info, BitFlags::all())
        .map(|sample_data| {
            // wi: incoming way direction
            let wi = sample_data.get_wi().normalize();
            let sampling_type = sample_data.get_sampling_type();
            let color = bsdf.eval(&wi, &wo, intersect_info, texture_list);

            // BSDF returns the incoming ray direction at the point of
            // intersection but for the next ray that is shot in the opposite
            // direction (into the scene), thus need to take the inverse of
            // `wi`.
            let next_ray_dir = -wi;

            ScatterHitData::new(
                color,
                Ray::new(*intersect_info.get_point(), next_ray_dir),
                sampling_type,
            )
        });

    let emission_data = bsdf
        .emission(&wo, mediums, intersect_info, texture_list)
        .map(EmissionHitData::new);

    (scattering_data, emission_data)
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
#[allow(clippy::too_many_arguments)]
pub fn trace_ray(
    ray: &Ray,
    camera: &Camera,
    scene: &Scene,
    depth: usize,
    shader_list: &ShaderList,
    texture_list: &TextureList,
    environment: &EnvironmentShadingData,
    mediums: &mut Mediums,
) -> (glm::DVec3, TraversalInfo) {
    if depth == 0 {
        return (glm::zero(), TraversalInfo::new());
    }

    let mut traversal_info = TraversalInfo::new();

    if let Some(info) = scene.hit(ray, 0.01, 1000.0) {
        let (scattering_data, emission_data) =
            shade_hit(ray, &info, shader_list, texture_list, mediums);

        // compute scattering of light
        let scattering_intensity = scattering_data.map_or(glm::zero(), |scattering_data| {
            let (traced_color, scatter_traversal_info) = trace_ray(
                &scattering_data.next_ray,
                camera,
                scene,
                depth - 1,
                shader_list,
                texture_list,
                environment,
                mediums,
            );

            traversal_info.append_traversal(scatter_traversal_info);

            glm::vec3(
                scattering_data.color[0] * traced_color[0],
                scattering_data.color[1] * traced_color[1],
                scattering_data.color[2] * traced_color[2],
            )
        });

        // compute emission of light
        let emission_intensity =
            emission_data.map_or(glm::zero(), |emission_data| emission_data.emission_color);

        // emission added to the scattered light
        let resulting_intensity = emission_intensity + scattering_intensity;

        // TODO: compute light fall off, it is not as simple as
        // resulting_intensity / (1.0 + info.get_t() * info.get_t())
        //
        // consider I0 to be the intensity at distance 0 and I1 to be
        // the intensity at distance 1. Now if we compute I1, I1 = I0
        // / (1.0 + 1.0 * 1.0) = I0 / 2
        //
        // Now if we compute I2 using I0 as the starting point, I2 =
        // I0 / (1.0 + 2.0 * 2.0) = I0 / 5.0
        //
        // If we consider I1 as the starting point, I2 = I1 / (1.0 +
        // 1.0 * 1.0) = I1 / 2.0 = I0 / 4.0 which is not the same as
        // the value that we calculated earlier
        //
        // So from what it looks like, the total distance to the
        // source of light (emission shaders in this case) must be
        // used to compute the color. This is easier said than done,
        // so will tackle this later.

        traversal_info.add_ray(SingleRayInfo::new(
            *ray,
            Some(*info.get_point()),
            resulting_intensity,
            Some(info.get_normal().unwrap()),
        ));

        (resulting_intensity, traversal_info)
    } else {
        let final_intensity = shade_environment(ray, environment);

        traversal_info.add_ray(SingleRayInfo::new(*ray, None, final_intensity, None));

        (final_intensity, traversal_info)
    }
}
