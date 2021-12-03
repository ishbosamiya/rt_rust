use crate::glm;

/// Get offset to struct member, similar to `offset_of` in C/C++
/// From <https://stackoverflow.com/questions/40310483/how-to-get-pointer-offset-in-bytes/40310851#40310851>
#[macro_export]
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(std::ptr::null() as *const $ty)).$field as *const _ as usize
    };
}

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
