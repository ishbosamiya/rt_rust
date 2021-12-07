use nalgebra::RealField;

use crate::glm;

/// str to CStr
pub fn str_to_cstr(string: &str) -> &std::ffi::CStr {
    return std::ffi::CStr::from_bytes_with_nul(string.as_bytes())
        .expect("ensure there is a '\\0' at the end of the string");
}

fn append_one(vec: &glm::DVec3) -> glm::DVec4 {
    glm::vec4(vec[0], vec[1], vec[2], 1.0)
}

pub fn vec2_apply_model_matrix(v: &glm::DVec2, model: &glm::DMat4) -> glm::DVec3 {
    glm::vec4_to_vec3(&(model * append_one(&glm::vec2_to_vec3(v))))
}

pub fn vec3_apply_model_matrix(v: &glm::DVec3, model: &glm::DMat4) -> glm::DVec3 {
    glm::vec4_to_vec3(&(model * append_one(v)))
}

pub fn normal_apply_model_matrix(normal: &glm::DVec3, model: &glm::DMat4) -> glm::DVec3 {
    vec3_apply_model_matrix(normal, &glm::inverse_transpose(*model))
}

pub fn focal_length_to_fov(focal_length: f64, camera_sensor_size: f64) -> f64 {
    2.0 * (camera_sensor_size / (2.0 * focal_length)).atan()
}

pub fn fov_to_focal_length(fov: f64, camera_sensor_size: f64) -> f64 {
    camera_sensor_size / (2.0 * (fov / 2.0).tan())
}

pub fn duration_to_string(duration: std::time::Duration) -> String {
    let time_taken = duration.as_secs_f64();
    if time_taken / 60.0 < 1.0 {
        format!("{:.3}s", time_taken)
    } else if time_taken / 60.0 / 60.0 < 1.0 {
        format!("{:.0}m {:.2}s", time_taken / 60.0, time_taken % 60.0)
    } else {
        format!(
            "{:.0}h {:.0}m {:.2}s",
            time_taken / 60.0,
            (time_taken / 60.0) % 60.0,
            ((time_taken / 60.0) % 60.0 / 60.0) % 60.0,
        )
    }
}

pub fn vec3_apply_bary_coord(
    v1: &glm::DVec3,
    v2: &glm::DVec3,
    v3: &glm::DVec3,
    bary_coord: &glm::DVec3,
) -> glm::DVec3 {
    v1 * bary_coord[0] + v2 * bary_coord[1] + v3 * bary_coord[2]
}

pub fn vec2_apply_bary_coord(
    v1: &glm::DVec2,
    v2: &glm::DVec2,
    v3: &glm::DVec2,
    bary_coord: &glm::DVec3,
) -> glm::DVec2 {
    v1 * bary_coord[0] + v2 * bary_coord[1] + v3 * bary_coord[2]
}

/// convert linear rgb to srgb
///
/// `linear`: rgb linear values between 0.0 and 1.0
///
/// Takes the first 3 values of `linear` and converts to srgb. `R` must be >= 3.
///
/// reference: https://en.wikipedia.org/wiki/SRGB#From_CIE_XYZ_to_sRGB
pub fn linear_to_srgb<const R: usize>(linear: &glm::TVec<f64, R>) -> glm::TVec<f64, R> {
    debug_assert!(R >= 3);

    let srgbize = |linear: f64| {
        // if linear <= 0.0031308 {
        //     12.92 * linear
        // } else {
        //     1.055 * linear.powf(1.0 / 2.4) - 0.055
        // }
        egui_glfw::egui::color::gamma_from_linear(linear as f32) as _
    };

    let mut srgb = *linear;
    srgb[0] = srgbize(srgb[0]);
    srgb[1] = srgbize(srgb[1]);
    srgb[2] = srgbize(srgb[2]);
    srgb
}

/// convert srgb to linear rgb
///
/// /// `srgb`: srgb values between 0.0 and 1.0
///
/// reference: https://en.wikipedia.org/wiki/SRGB#From_sRGB_to_CIE_XYZ
pub fn srgb_to_linear<T: RealField + simba::scalar::SubsetOf<f32>, const R: usize>(
    srgb: &glm::TVec<T, R>,
) -> glm::TVec<T, R> {
    let linearize = |srgb: T| {
        // if srgb <= 0.04045 {
        //     srgb / 12.92
        // } else {
        //     ((srgb + 0.055) / 1.055).powf(2.4)
        // }
        T::from_f32(egui_glfw::egui::color::linear_from_gamma(glm::convert(
            srgb,
        )))
        .unwrap()
    };

    let mut linear = *srgb;
    linear[0] = linearize(linear[0]);
    linear[1] = linearize(linear[1]);
    linear[2] = linearize(linear[2]);
    linear
}
