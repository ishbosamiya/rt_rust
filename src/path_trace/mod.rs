pub mod bsdf;
pub mod bsdfs;
pub mod camera;
pub mod intersectable;
pub mod ray;
pub mod shader_list;
pub mod shaders;
pub mod traversal_info;

use enumflags2::BitFlags;

use crate::glm;
use crate::path_trace::bsdf::SamplingTypes;
use crate::path_trace::camera::Camera;
use crate::path_trace::intersectable::IntersectInfo;
use crate::path_trace::intersectable::Intersectable;
use crate::path_trace::ray::Ray;
use crate::scene::Scene;

use self::shader_list::ShaderList;
use self::traversal_info::SingleRayInfo;
use self::traversal_info::TraversalInfo;

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
    let shader = shader_list
        .get_shader(intersect_info.get_shader_id().unwrap())
        .unwrap()
        .get_bsdf();

    // wo: outgoing ray direction
    //
    // Outgoing ray direction must be the inverse of the current ray since
    // the current ray are travelling from camera into the scene and the
    // BSDF need not care about that. It must receive only the outgoing
    // direction.
    let wo = -ray.get_direction();

    // wi: incoming way direction
    let op_sample_data = shader.sample(ray.get_direction(), intersect_info, BitFlags::all());

    if let Some(sample_data) = op_sample_data {
        let wi = sample_data.get_wi();
        let sampling_type = sample_data.get_sampling_type();

        // BSDF returns the incoming ray direction at the point of
        // intersection but for the next ray that is shot in the opposite
        // direction (into the scene), thus need to take the inverse of
        // `wi`.
        let wi = -wi;

        let color = shader.eval(&wi, &wo, intersect_info);
        let emission = shader.emission(intersect_info);
        if let Some(emission) = emission {
            ShadeHitData::Both(ShadeHitDataBoth::new(
                color,
                emission,
                Ray::new(*intersect_info.get_point(), wi),
                sampling_type,
            ))
        } else {
            ShadeHitData::ScatterOnly(ShadeHitDataScatterOnly::new(
                color,
                Ray::new(*intersect_info.get_point(), wi),
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
