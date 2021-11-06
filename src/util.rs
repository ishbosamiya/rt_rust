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
