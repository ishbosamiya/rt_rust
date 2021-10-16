pub mod bsdf;
pub mod bsdfs;
pub mod camera;
pub mod intersectable;
pub mod ray;
pub mod shader_list;
pub mod shaders;

use enumflags2::BitFlags;

use crate::glm;
use crate::path_trace::bsdf::SamplingTypes;
use crate::path_trace::camera::Camera;
use crate::path_trace::intersectable::IntersectInfo;
use crate::path_trace::intersectable::Intersectable;
use crate::path_trace::ray::Ray;
use crate::scene::Scene;

use self::shader_list::ShaderList;

/// Data that is returned during the `shade_hit()` calculation
#[derive(Debug, Clone, PartialEq)]
pub struct ShadeHitData {
    /// color information that should be propagated forward
    color: glm::DVec3,
    /// the next ray to continue the ray tracing, calculated from the
    /// `BSDF`
    next_ray: Ray,
    /// type of sampling performed to generate the next ray by the
    /// `BSDF`
    sampling_type: SamplingTypes,
}

impl ShadeHitData {
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

fn shade_environment(ray: &Ray, camera: &Camera) -> glm::DVec3 {
    let color_1 = glm::vec3(0.8, 0.8, 0.8);
    let color_2 = glm::vec3(0.2, 0.2, 0.8);

    let camera_origin_y = camera.get_origin()[1];
    let camera_vertical_range = camera.get_vertical()[1];
    let y_val = (camera_origin_y + ray.get_direction()[1]) / camera_vertical_range;
    let y_val = (y_val + 1.0) / 2.0;

    glm::lerp(&color_1, &color_2, y_val)
}

/// Shade the point of intersection when the ray hits an object
fn shade_hit(ray: &Ray, intersect_info: &IntersectInfo) -> ShadeHitData {
    let mut shader_list = ShaderList::new();
    let lambert_shader_id = shader_list.add_shader(Box::new(shaders::Lambert::new(
        bsdfs::lambert::Lambert::new(glm::vec4(0.0, 1.0, 1.0, 1.0)),
    )));
    let glossy_shader_id = shader_list.add_shader(Box::new(shaders::Glossy::new(
        bsdfs::glossy::Glossy::new(glm::vec4(0.0, 1.0, 1.0, 1.0)),
    )));

    // TODO: need to remove `roughness_amount`, it is only a test for
    // `ShaderList` right now, it mixes lambert and glossy shader.
    //
    // TODO: `shader` must be got from the `ShaderID` stored
    // `IntersectInfo`.
    let roughness_amount = 0.2;
    let shader = if rand::random::<f64>() < roughness_amount {
        shader_list
            .get_shader(lambert_shader_id)
            .unwrap()
            .get_bsdf()
    } else {
        shader_list.get_shader(glossy_shader_id).unwrap().get_bsdf()
    };

    // wo: outgoing ray direction
    //
    // Outgoing ray direction must be the inverse of the current ray since
    // the current ray are travelling from camera into the scene and the
    // BSDF need not care about that. It must receive only the outgoing
    // direction.
    let wo = -ray.get_direction();

    // wi: incoming way direction
    let sample_data = shader
        .sample(ray.get_direction(), intersect_info, BitFlags::all())
        .expect("todo: need to handle the case where the sample returns None");

    let wi = sample_data.get_wi();
    let sampling_type = sample_data.get_sampling_type();

    // BSDF returns the incoming ray direction at the point of
    // intersection but for the next ray that is shot in the opposite
    // direction (into the scene), thus need to take the inverse of
    // `wi`.
    let wi = -wi;

    let color = shader.eval(&wi, &wo, intersect_info);
    ShadeHitData::new(
        color,
        Ray::new(*intersect_info.get_point(), wi),
        sampling_type,
    )
}

// x: current point
// x_prime: previous point
// x_prime_prime: previous's previous point
// g: geometry term, 1/(r^2) where r is distance of x_prime to x
// e: intensity of emitted light by x_prime reaching x
// i: intensity of light from x_prime to x
// p: intensity of light scattered from x_prime_prime to x by a patch on surface at x_prime
pub fn trace_ray(ray: &Ray, camera: &Camera, scene: &Scene, depth: usize) -> glm::DVec3 {
    if depth == 0 {
        return glm::zero();
    }
    let val;
    if let Some(info) = scene.hit(ray, 0.01, 1000.0) {
        let ShadeHitData {
            color,
            next_ray,
            sampling_type: _,
        } = shade_hit(ray, &info);
        let traced_color = trace_ray(&next_ray, camera, scene, depth - 1);
        val = glm::vec3(
            color[0] * traced_color[0],
            color[1] * traced_color[1],
            color[2] * traced_color[2],
        );
    } else {
        val = shade_environment(ray, camera);
    }
    val
}
