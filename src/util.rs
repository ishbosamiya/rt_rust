use egui_glfw::egui;
use lazy_static::lazy_static;

use std::{convert::TryFrom, fmt::Display};

use crate::{glm, ui::DrawUI};

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
/// reference: <https://en.wikipedia.org/wiki/SRGB#From_CIE_XYZ_to_sRGB>
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
/// reference: <https://en.wikipedia.org/wiki/SRGB#From_sRGB_to_CIE_XYZ>
pub fn srgb_to_linear<const R: usize>(srgb: &glm::TVec<f64, R>) -> glm::TVec<f64, R> {
    let linearize = |srgb: f64| {
        // if srgb <= 0.04045 {
        //     srgb / 12.92
        // } else {
        //     ((srgb + 0.055) / 1.055).powf(2.4)
        // }
        egui_glfw::egui::color::linear_from_gamma(srgb as f32) as _
    };

    let mut linear = *srgb;
    linear[0] = linearize(linear[0]);
    linear[1] = linearize(linear[1]);
    linear[2] = linearize(linear[2]);
    linear
}

/// Convert normal represented in a slice of i16 to glm::DVec3
///
/// This is based on Blender's `normal_short_to_float_v3()` function
/// in the `file math_vector_inline.c`.
pub fn normal_i16_slice_to_dvec3(normal: &[i16]) -> glm::DVec3 {
    glm::vec3(
        normal[0] as f64 * (1.0 / 32767.0),
        normal[1] as f64 * (1.0 / 32767.0),
        normal[2] as f64 * (1.0 / 32767.0),
    )
}

/// Checks if the file header contains the magic bytes to represent a
/// gzip file
///
/// From Blender's `BLI_file_magic_is_gzip()` in `fileops.c`
pub fn file_magic_is_gzip(data: &[u8]) -> bool {
    // GZIP itself starts with the magic bytes 0x1f 0x8b. The third
    // byte indicates the compression method, which is 0x08 for
    // DEFLATE.
    data[0] == 0x1f && data[1] == 0x8b && data[2] == 0x08
}

/// Checks if the file header contains the magic bytes to represent a
/// zstd file
///
/// From Blender's `BLI_file_magic_is_zstd()` in `fileops.c`
pub fn file_magic_is_zstd(data: &[u8]) -> bool {
    // ZSTD files consist of concatenated frames, each either a Zstd
    // frame or a skippable frame.  Both types of frames start with a
    // magic number: 0xFD2FB528 for Zstd frames and 0x184D2A5* for
    // skippable frames, with the * being anything from 0 to F.
    //
    // To check whether a file is Zstd-compressed, we just check
    // whether the first frame matches either. Seeking through the
    // file until a Zstd frame is found would make things more
    // complicated and the probability of a false positive is rather
    // low anyways.
    //
    // Note that LZ4 uses a compatible format, so even though its
    // compressed frames have a different magic number, a valid LZ4
    // file might also start with a skippable frame matching the
    // second check here.
    //
    // For more details, see
    // https://github.com/facebook/zstd/blob/dev/doc/zstd_compression_format.md

    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

    magic == 0xFD2FB528 || (magic >> 4) == 0x184D2A5
}

// Axis conversion code is based on Blender's io_utils.py,
// axis_conversion() function

lazy_static! {
    /// The "or"ed values of the from_forward, from_up, to_forward, to_up.
    static ref AXIS_CONVERT_LUT: Vec<Vec<usize>> = {
        vec![
            vec![
                0x8C8, 0x4D0, 0x2E0, 0xAE8, 0x701, 0x511, 0x119, 0xB29, 0x682, 0x88A, 0x09A, 0x2A2,
                0x80B, 0x413, 0x223, 0xA2B, 0x644, 0x454, 0x05C, 0xA6C, 0x745, 0x94D, 0x15D, 0x365,
            ],
            vec![
                0xAC8, 0x8D0, 0x4E0, 0x2E8, 0x741, 0x951, 0x159, 0x369, 0x702, 0xB0A, 0x11A, 0x522,
                0xA0B, 0x813, 0x423, 0x22B, 0x684, 0x894, 0x09C, 0x2AC, 0x645, 0xA4D, 0x05D, 0x465,
            ],
            vec![
                0x4C8, 0x2D0, 0xAE0, 0x8E8, 0x681, 0x291, 0x099, 0x8A9, 0x642, 0x44A, 0x05A, 0xA62,
                0x40B, 0x213, 0xA23, 0x82B, 0x744, 0x354, 0x15C, 0x96C, 0x705, 0x50D, 0x11D, 0xB25,
            ],
            vec![
                0x2C8, 0xAD0, 0x8E0, 0x4E8, 0x641, 0xA51, 0x059, 0x469, 0x742, 0x34A, 0x15A, 0x962,
                0x20B, 0xA13, 0x823, 0x42B, 0x704, 0xB14, 0x11C, 0x52C, 0x685, 0x28D, 0x09D, 0x8A5,
            ],
            vec![
                0x708, 0xB10, 0x120, 0x528, 0x8C1, 0xAD1, 0x2D9, 0x4E9, 0x942, 0x74A, 0x35A, 0x162,
                0x64B, 0xA53, 0x063, 0x46B, 0x804, 0xA14, 0x21C, 0x42C, 0x885, 0x68D, 0x29D, 0x0A5,
            ],
            vec![
                0xB08, 0x110, 0x520, 0x728, 0x941, 0x151, 0x359, 0x769, 0x802, 0xA0A, 0x21A, 0x422,
                0xA4B, 0x053, 0x463, 0x66B, 0x884, 0x094, 0x29C, 0x6AC, 0x8C5, 0xACD, 0x2DD, 0x4E5,
            ],
            vec![
                0x508, 0x710, 0xB20, 0x128, 0x881, 0x691, 0x299, 0x0A9, 0x8C2, 0x4CA, 0x2DA, 0xAE2,
                0x44B, 0x653, 0xA63, 0x06B, 0x944, 0x754, 0x35C, 0x16C, 0x805, 0x40D, 0x21D, 0xA25,
            ],
            vec![
                0x108, 0x510, 0x720, 0xB28, 0x801, 0x411, 0x219, 0xA29, 0x882, 0x08A, 0x29A, 0x6A2,
                0x04B, 0x453, 0x663, 0xA6B, 0x8C4, 0x4D4, 0x2DC, 0xAEC, 0x945, 0x14D, 0x35D, 0x765,
            ],
            vec![
                0x748, 0x350, 0x160, 0x968, 0xAC1, 0x2D1, 0x4D9, 0x8E9, 0xA42, 0x64A, 0x45A, 0x062,
                0x68B, 0x293, 0x0A3, 0x8AB, 0xA04, 0x214, 0x41C, 0x82C, 0xB05, 0x70D, 0x51D, 0x125,
            ],
            vec![
                0x948, 0x750, 0x360, 0x168, 0xB01, 0x711, 0x519, 0x129, 0xAC2, 0x8CA, 0x4DA, 0x2E2,
                0x88B, 0x693, 0x2A3, 0x0AB, 0xA44, 0x654, 0x45C, 0x06C, 0xA05, 0x80D, 0x41D, 0x225,
            ],
            vec![
                0x348, 0x150, 0x960, 0x768, 0xA41, 0x051, 0x459, 0x669, 0xA02, 0x20A, 0x41A, 0x822,
                0x28B, 0x093, 0x8A3, 0x6AB, 0xB04, 0x114, 0x51C, 0x72C, 0xAC5, 0x2CD, 0x4DD, 0x8E5,
            ],
            vec![
                0x148, 0x950, 0x760, 0x368, 0xA01, 0x811, 0x419, 0x229, 0xB02, 0x10A, 0x51A, 0x722,
                0x08B, 0x893, 0x6A3, 0x2AB, 0xAC4, 0x8D4, 0x4DC, 0x2EC, 0xA45, 0x04D, 0x45D, 0x665,
            ],
            vec![
                0x688, 0x890, 0x0A0, 0x2A8, 0x4C1, 0x8D1, 0xAD9, 0x2E9, 0x502, 0x70A, 0xB1A, 0x122,
                0x74B, 0x953, 0x163, 0x36B, 0x404, 0x814, 0xA1C, 0x22C, 0x445, 0x64D, 0xA5D, 0x065,
            ],
            vec![
                0x888, 0x090, 0x2A0, 0x6A8, 0x501, 0x111, 0xB19, 0x729, 0x402, 0x80A, 0xA1A, 0x222,
                0x94B, 0x153, 0x363, 0x76B, 0x444, 0x054, 0xA5C, 0x66C, 0x4C5, 0x8CD, 0xADD, 0x2E5,
            ],
            vec![
                0x288, 0x690, 0x8A0, 0x0A8, 0x441, 0x651, 0xA59, 0x069, 0x4C2, 0x2CA, 0xADA, 0x8E2,
                0x34B, 0x753, 0x963, 0x16B, 0x504, 0x714, 0xB1C, 0x12C, 0x405, 0x20D, 0xA1D, 0x825,
            ],
            vec![
                0x088, 0x290, 0x6A0, 0x8A8, 0x401, 0x211, 0xA19, 0x829, 0x442, 0x04A, 0xA5A, 0x662,
                0x14B, 0x353, 0x763, 0x96B, 0x4C4, 0x2D4, 0xADC, 0x8EC, 0x505, 0x10D, 0xB1D, 0x725,
            ],
            vec![
                0x648, 0x450, 0x060, 0xA68, 0x2C1, 0x4D1, 0x8D9, 0xAE9, 0x282, 0x68A, 0x89A, 0x0A2,
                0x70B, 0x513, 0x123, 0xB2B, 0x204, 0x414, 0x81C, 0xA2C, 0x345, 0x74D, 0x95D, 0x165,
            ],
            vec![
                0xA48, 0x650, 0x460, 0x068, 0x341, 0x751, 0x959, 0x169, 0x2C2, 0xACA, 0x8DA, 0x4E2,
                0xB0B, 0x713, 0x523, 0x12B, 0x284, 0x694, 0x89C, 0x0AC, 0x205, 0xA0D, 0x81D, 0x425,
            ],
            vec![
                0x448, 0x050, 0xA60, 0x668, 0x281, 0x091, 0x899, 0x6A9, 0x202, 0x40A, 0x81A, 0xA22,
                0x50B, 0x113, 0xB23, 0x72B, 0x344, 0x154, 0x95C, 0x76C, 0x2C5, 0x4CD, 0x8DD, 0xAE5,
            ],
            vec![
                0x048, 0xA50, 0x660, 0x468, 0x201, 0xA11, 0x819, 0x429, 0x342, 0x14A, 0x95A, 0x762,
                0x10B, 0xB13, 0x723, 0x52B, 0x2C4, 0xAD4, 0x8DC, 0x4EC, 0x285, 0x08D, 0x89D, 0x6A5,
            ],
            vec![
                0x808, 0xA10, 0x220, 0x428, 0x101, 0xB11, 0x719, 0x529, 0x142, 0x94A, 0x75A, 0x362,
                0x8CB, 0xAD3, 0x2E3, 0x4EB, 0x044, 0xA54, 0x65C, 0x46C, 0x085, 0x88D, 0x69D, 0x2A5,
            ],
            vec![
                0xA08, 0x210, 0x420, 0x828, 0x141, 0x351, 0x759, 0x969, 0x042, 0xA4A, 0x65A, 0x462,
                0xACB, 0x2D3, 0x4E3, 0x8EB, 0x084, 0x294, 0x69C, 0x8AC, 0x105, 0xB0D, 0x71D, 0x525,
            ],
            vec![
                0x408, 0x810, 0xA20, 0x228, 0x081, 0x891, 0x699, 0x2A9, 0x102, 0x50A, 0x71A, 0xB22,
                0x4CB, 0x8D3, 0xAE3, 0x2EB, 0x144, 0x954, 0x75C, 0x36C, 0x045, 0x44D, 0x65D, 0xA65,
            ],
        ]
    };

    /// The conversion matricies
    ///
    /// The matrices correspond exactly with the AXIS_CONVERT_LUT positionally.
    static ref AXIS_CONVERT_MATRIX: Vec<glm::DMat3> = {
        vec![
            glm::mat3(-1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0),
            glm::mat3(-1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, -1.0, 0.0),
            glm::mat3(-1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0),
            glm::mat3(-1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0),
            glm::mat3(0.0, -1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, -1.0),
            glm::mat3(0.0, 0.0, 1.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0),
            glm::mat3(0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0),
            glm::mat3(0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0),
            glm::mat3(0.0, -1.0, 0.0, 0.0, 0.0, 1.0, -1.0, 0.0, 0.0),
            glm::mat3(0.0, 0.0, -1.0, 0.0, -1.0, 0.0, -1.0, 0.0, 0.0),
            glm::mat3(0.0, 0.0, 1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0),
            glm::mat3(0.0, 1.0, 0.0, 0.0, 0.0, -1.0, -1.0, 0.0, 0.0),
            glm::mat3(0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 1.0, 0.0, 0.0),
            glm::mat3(0.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, 0.0),
            glm::mat3(0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0),
            glm::mat3(0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0),
            glm::mat3(0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0),
            glm::mat3(0.0, 0.0, -1.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0),
            glm::mat3(0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0),
            glm::mat3(0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.0),
            glm::mat3(1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0),
            glm::mat3(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0),
            glm::mat3(1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0),
        ]
    };
}

/// 3D axis directions
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Axis {
    X,
    Y,
    Z,
    NegX,
    NegY,
    NegZ,
}

impl TryFrom<usize> for Axis {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, ()> {
        match value {
            0 => Ok(Axis::X),
            1 => Ok(Axis::Y),
            2 => Ok(Axis::Z),
            3 => Ok(Axis::NegX),
            4 => Ok(Axis::NegY),
            5 => Ok(Axis::NegZ),
            _ => Err(()),
        }
    }
}

impl From<Axis> for usize {
    fn from(axis: Axis) -> Self {
        match axis {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
            Axis::NegX => 3,
            Axis::NegY => 4,
            Axis::NegZ => 5,
        }
    }
}

impl Display for Axis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Axis::X => write!(f, "X"),
            Axis::Y => write!(f, "Y"),
            Axis::Z => write!(f, "Z"),
            Axis::NegX => write!(f, "-X"),
            Axis::NegY => write!(f, "-Y"),
            Axis::NegZ => write!(f, "-Z"),
        }
    }
}

impl Axis {
    pub fn all() -> impl Iterator<Item = Self> {
        use Axis::*;
        [X, Y, Z, NegX, NegY, NegZ].iter().copied()
    }
}

impl DrawUI for Axis {
    type ExtraData = egui::Id;

    fn draw_ui(&self, _ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {
        unreachable!("no non mut draw ui for InfoType")
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui, id: &Self::ExtraData) {
        egui::ComboBox::from_id_source(id.with("Axis"))
            .selected_text(format!("{}", self))
            .show_ui(ui, |ui| {
                Self::all().for_each(|info| {
                    ui.selectable_value(self, info, format!("{}", info));
                });
            });
    }
}

/// Convert between different axis setups, returns None if conversion
/// not possible.
///
/// Forward and Up vectors are enough to determine the entire
/// coordinate axis system. So given forward and up vectors to convert
/// from, the function returns a model matrix that can be applied to
/// the object.
///
/// In this case, we are considering Blender's coordinate axis
/// convention for what is considered positive X, Y, Z
///
/// X: screen left to right is positive
///
/// Y: into the screen is positive
///
/// Z: screen bottom to top is positive
///
/// Conversion not possible when `from_forward == from_up` or
/// `to_forward` == `to_up`.
pub fn axis_conversion_matrix(
    from_forward: Axis,
    from_up: Axis,
    to_forward: Axis,
    to_up: Axis,
) -> Option<glm::DMat4> {
    if from_forward == to_forward && from_up == to_up {
        return Some(glm::identity());
    }

    if from_forward == from_up || to_forward == to_up {
        return None;
    }

    let from_to: [usize; 4] = [
        from_forward.into(),
        from_up.into(),
        to_forward.into(),
        to_up.into(),
    ];

    let value: usize = from_to
        .iter()
        .enumerate()
        .fold(0, |acc, (i, axis)| acc | (axis << (i * 3)));

    for (i, axis_lut) in AXIS_CONVERT_LUT.iter().enumerate() {
        if axis_lut.contains(&value) {
            return Some(glm::mat3_to_mat4(&AXIS_CONVERT_MATRIX[i]));
        }
    }
    // any configuration that is not valid, the forward and up axis
    // are not perpendicular to each other
    None
}

/// Axis conversion matrix to convert axis from Blender to X as right,
/// Y as up with right hand thumb rule (OpenGL axis).
pub fn axis_conversion_matrix_from_blender() -> glm::DMat4 {
    axis_conversion_matrix(Axis::Y, Axis::Z, Axis::NegZ, Axis::Y).unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationModes {
    Quaternion,
    EulerXYZ,
    EulerXZY,
    EulerYXZ,
    EulerYZX,
    EulerZXY,
    EulerZYX,
    AxisAngle,
    // TODO: need to finalize the axis by which roll, pitch and yaw
    // are defined, right now it is based on
    // https://learnopengl.com/Getting-started/Camera which considers
    // camera pitch to be up down (x axis) and camera yaw to be left
    // right (y axis). No roll is defined so until finalized, roll is
    // always zero :(
    RollPitchYaw,
}

impl RotationModes {
    pub fn all() -> impl Iterator<Item = Self> {
        use RotationModes::*;
        [
            Quaternion,
            EulerXYZ,
            EulerXZY,
            EulerYXZ,
            EulerYZX,
            EulerZXY,
            EulerZYX,
            AxisAngle,
            RollPitchYaw,
        ]
        .iter()
        .copied()
    }
}

impl Display for RotationModes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RotationModes::Quaternion => write!(f, "Quaternion"),
            RotationModes::EulerXYZ => write!(f, "Euler XYZ"),
            RotationModes::EulerXZY => write!(f, "Euler XZY"),
            RotationModes::EulerYXZ => write!(f, "Euler YXZ"),
            RotationModes::EulerYZX => write!(f, "Euler YZX"),
            RotationModes::EulerZXY => write!(f, "Euler ZXY"),
            RotationModes::EulerZYX => write!(f, "Euler ZYX"),
            RotationModes::AxisAngle => write!(f, "Axis Angle"),
            RotationModes::RollPitchYaw => write!(f, "Roll Pitch Yaw"),
        }
    }
}

impl DrawUI for RotationModes {
    type ExtraData = egui::Id;

    fn draw_ui(&self, _ui: &mut egui::Ui, _extra_data: &Self::ExtraData) {
        unreachable!("no non mut draw ui for InfoType")
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui, id: &Self::ExtraData) {
        egui::ComboBox::from_id_source(id.with("Rotation Mode"))
            .selected_text(format!("{}", self))
            .show_ui(ui, |ui| {
                Self::all().for_each(|info| {
                    ui.selectable_value(self, info, format!("{}", info));
                });
            });
    }
}

/// Convert the given euler rotation to a rotation matrix
///
/// reference:
/// <https://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.371.6578>
pub fn euler_to_rotation_matrix(rot: &glm::DVec3, mode: RotationModes) -> glm::DMat3 {
    let (psi, theta, phi) = (rot[0], rot[1], rot[2]);

    let cos_psi = psi.cos();
    let sin_psi = psi.sin();
    let rx = glm::mat3(1.0, 0.0, 0.0, 0.0, cos_psi, -sin_psi, 0.0, sin_psi, cos_psi);

    let cos_theta = theta.cos();
    let sin_theta = theta.sin();
    let ry = glm::mat3(
        cos_theta, 0.0, sin_theta, 0.0, 1.0, 0.0, -sin_theta, 0.0, cos_theta,
    );

    let cos_phi = phi.cos();
    let sin_phi = phi.sin();
    let rz = glm::mat3(cos_phi, -sin_phi, 0.0, sin_phi, cos_phi, 0.0, 0.0, 0.0, 1.0);

    match mode {
        RotationModes::Quaternion => todo!(),
        RotationModes::EulerXYZ => rx * ry * rz,
        RotationModes::EulerXZY => rx * rz * ry,
        RotationModes::EulerYXZ => ry * rx * rz,
        RotationModes::EulerYZX => ry * rz * rx,
        RotationModes::EulerZXY => rz * rx * ry,
        RotationModes::EulerZYX => rz * ry * rx,
        RotationModes::AxisAngle => todo!(),
        RotationModes::RollPitchYaw => todo!(),
    }
}

/// Convert the given rotation matrix to a Euler rotation for the given mode
///
/// reference: <https://www.andre-gaschler.com/rotationconverter/>
/// <https://threejs.org/docs/index.html?q=eule#api/en/math/Euler>
/// <https://github.com/mrdoob/three.js/blob/master/src/math/Euler.js>
pub fn rotation_matrix_to_euler(mat: &glm::DMat3, mode: RotationModes) -> glm::DVec3 {
    let very_close_to_one = 0.99999999999;
    match mode {
        RotationModes::Quaternion => todo!(),
        RotationModes::EulerXYZ => {
            let y = mat.index((0, 2)).clamp(-1.0, 1.0).asin();
            let (x, z) = if *mat.index((0, 2)) < very_close_to_one {
                (
                    (-mat.index((1, 2))).atan2(*mat.index((2, 2))),
                    (-mat.index((0, 1))).atan2(*mat.index((0, 0))),
                )
            } else {
                (mat.index((2, 1)).atan2(*mat.index((1, 1))), 0.0)
            };
            glm::vec3(x, y, z)
        }
        RotationModes::EulerXZY => {
            let z = (-mat.index((0, 1)).clamp(-1.0, 1.0)).asin();
            let (x, y) = if *mat.index((0, 1)) < very_close_to_one {
                (
                    mat.index((2, 1)).atan2(*mat.index((1, 1))),
                    mat.index((0, 2)).atan2(*mat.index((0, 0))),
                )
            } else {
                ((-mat.index((1, 2))).atan2(*mat.index((2, 2))), 0.0)
            };
            glm::vec3(x, y, z)
        }
        RotationModes::EulerYXZ => {
            let x = (-mat.index((1, 2)).clamp(-1.0, 1.0)).asin();
            let (y, z) = if *mat.index((1, 2)) < very_close_to_one {
                (
                    mat.index((0, 2)).atan2(*mat.index((2, 2))),
                    mat.index((1, 0)).atan2(*mat.index((1, 1))),
                )
            } else {
                ((-mat.index((2, 0))).atan2(*mat.index((0, 0))), 0.0)
            };
            glm::vec3(x, y, z)
        }
        RotationModes::EulerYZX => {
            let z = mat.index((1, 0)).clamp(-1.0, 1.0).asin();
            let (x, y) = if *mat.index((1, 0)) < very_close_to_one {
                (
                    (-mat.index((1, 2))).atan2(*mat.index((1, 1))),
                    (-mat.index((2, 0))).atan2(*mat.index((0, 0))),
                )
            } else {
                (0.0, mat.index((0, 2)).atan2(*mat.index((2, 2))))
            };
            glm::vec3(x, y, z)
        }
        RotationModes::EulerZXY => {
            let x = mat.index((2, 1)).clamp(-1.0, 1.0).asin();
            let (y, z) = if *mat.index((2, 1)) < very_close_to_one {
                (
                    (-mat.index((2, 0))).atan2(*mat.index((2, 2))),
                    (-mat.index((0, 1))).atan2(*mat.index((1, 1))),
                )
            } else {
                (0.0, mat.index((1, 0)).atan2(*mat.index((0, 0))))
            };
            glm::vec3(x, y, z)
        }
        RotationModes::EulerZYX => {
            let y = (-mat.index((2, 0))).clamp(-1.0, 1.0).asin();
            let (x, z) = if *mat.index((2, 0)) < very_close_to_one {
                (
                    mat.index((2, 1)).atan2(*mat.index((2, 2))),
                    mat.index((1, 0)).atan2(*mat.index((0, 0))),
                )
            } else {
                (0.0, (-mat.index((2, 0))).atan2(*mat.index((0, 0))))
            };
            glm::vec3(x, y, z)
        }
        RotationModes::AxisAngle => todo!(),
        RotationModes::RollPitchYaw => {
            // TODO: need to calculate roll, right now it is just
            // returning pitch and yaw which is based on
            // https://learnopengl.com/Getting-started/Camera

            // The weird thing about this is that there is no exact
            // notation, it is all very arbitrary. This considers the
            // pitch to be camera up down (x axis), yaw to be camera
            // left right (y axis).

            let pitch = mat.index((2, 1)).asin();
            let yaw = -(mat.index((2, 2))).atan2(*mat.index((2, 0)));
            glm::vec3(0.0, pitch, yaw)
        }
    }
}

/// Convert from one rotation mode to another
pub fn euler_rotation_change_mode(
    rot: &glm::DVec3,
    from: RotationModes,
    to: RotationModes,
) -> glm::DVec3 {
    let mat = euler_to_rotation_matrix(rot, from);
    rotation_matrix_to_euler(&mat, to)
}

#[macro_export]
macro_rules! assert_vec_relative_eq {
    ($left:expr, $right:expr, $epsilon:expr $(,)?) => {
        match (&$left, &$right, &$epsilon) {
            (left_val, right_val, epsilon) => {
                assert_eq!(left_val.len(), right_val.len());
                for i in 0..left_val.len() {
                    if (left_val[i] - right_val[i]).abs() >= *epsilon {
                        panic!("{:?} != {:?}", left_val, right_val);
                    }
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_conversion_matrix_test_01() {
        let conversion_matrix = axis_conversion_matrix(Axis::X, Axis::Y, Axis::X, Axis::Y).unwrap();

        let identity_mat4: glm::DMat4 = glm::identity();

        assert_eq!(identity_mat4, conversion_matrix);
    }

    #[test]
    fn euler_to_rotation_matrix_test_xyz() {
        let rot_mat = euler_to_rotation_matrix(
            &glm::vec3(
                50.0_f64.to_radians(),
                20.0_f64.to_radians(),
                -30.0_f64.to_radians(),
            ),
            RotationModes::EulerXYZ,
        );

        // expected value found from https://www.andre-gaschler.com/rotationconverter/
        assert_vec_relative_eq!(
            rot_mat,
            glm::mat3(
                0.8137977, 0.4698463, 0.3420202, -0.0944929, 0.6876717, -0.7198463, -0.5734147,
                0.5534908, 0.6040228
            ),
            1e-6
        );
    }

    #[test]
    fn euler_to_rotation_matrix_test_xzy() {
        let rot_mat = euler_to_rotation_matrix(
            &glm::vec3(
                50.0_f64.to_radians(),
                20.0_f64.to_radians(),
                -30.0_f64.to_radians(),
            ),
            RotationModes::EulerXZY,
        );

        // expected value found from https://www.andre-gaschler.com/rotationconverter/
        assert_vec_relative_eq!(
            rot_mat,
            glm::mat3(
                0.8137977, 0.5000000, 0.2961981, -0.0400088, 0.5566704, -0.8297695, -0.5797695,
                0.6634139, 0.4730214
            ),
            1e-6
        );
    }

    #[test]
    fn euler_to_rotation_matrix_test_yxz() {
        let rot_mat = euler_to_rotation_matrix(
            &glm::vec3(
                50.0_f64.to_radians(),
                20.0_f64.to_radians(),
                -30.0_f64.to_radians(),
            ),
            RotationModes::EulerYXZ,
        );

        // expected value found from https://www.andre-gaschler.com/rotationconverter/
        assert_vec_relative_eq!(
            rot_mat,
            glm::mat3(
                0.6827964, 0.6967472, 0.2198463, -0.3213938, 0.5566704, -0.7660444, -0.6561213,
                0.4523951, 0.6040228
            ),
            1e-6
        );
    }

    #[test]
    fn euler_to_rotation_matrix_test_yzx() {
        let rot_mat = euler_to_rotation_matrix(
            &glm::vec3(
                50.0_f64.to_radians(),
                20.0_f64.to_radians(),
                -30.0_f64.to_radians(),
            ),
            RotationModes::EulerYZX,
        );

        // expected value found from https://www.andre-gaschler.com/rotationconverter/
        assert_vec_relative_eq!(
            rot_mat,
            glm::mat3(
                0.8137977, 0.5640140, -0.1400768, -0.5000000, 0.5566704, -0.6634139, -0.2961981,
                0.6099232, 0.7350241
            ),
            1e-6
        );
    }

    #[test]
    fn euler_to_rotation_matrix_test_zxy() {
        let rot_mat = euler_to_rotation_matrix(
            &glm::vec3(
                50.0_f64.to_radians(),
                20.0_f64.to_radians(),
                -30.0_f64.to_radians(),
            ),
            RotationModes::EulerZXY,
        );

        // expected value found from https://www.andre-gaschler.com/rotationconverter/
        assert_vec_relative_eq!(
            rot_mat,
            glm::mat3(
                0.9447990, 0.3213938, -0.0637250, -0.2429454, 0.5566704, -0.7944152, -0.2198463,
                0.7660444, 0.6040228
            ),
            1e-6
        );
    }

    #[test]
    fn euler_to_rotation_matrix_test_zyx() {
        let rot_mat = euler_to_rotation_matrix(
            &glm::vec3(
                50.0_f64.to_radians(),
                20.0_f64.to_radians(),
                -30.0_f64.to_radians(),
            ),
            RotationModes::EulerZYX,
        );

        // expected value found from https://www.andre-gaschler.com/rotationconverter/
        assert_vec_relative_eq!(
            rot_mat,
            glm::mat3(
                0.8137977, 0.5482947, -0.1926297, -0.4698463, 0.4256691, -0.7733371, -0.3420202,
                0.7198463, 0.6040228
            ),
            1e-6
        );
    }

    macro_rules! test_rotation_matrix_to_euler_01 {
        ($test_name:ident , $rotation_mode:ident) => {
            #[test]
            fn $test_name() {
                let initial_euler = glm::vec3(
                    50.0_f64.to_radians(),
                    20.0_f64.to_radians(),
                    -30.0_f64.to_radians(),
                );
                let rot_mat =
                    euler_to_rotation_matrix(&initial_euler, RotationModes::$rotation_mode);

                let res = rotation_matrix_to_euler(&rot_mat, RotationModes::$rotation_mode);

                assert_vec_relative_eq!(res, initial_euler, 1e-6);
            }
        };
    }

    test_rotation_matrix_to_euler_01!(rotation_matrix_to_euler_test_xyz_01, EulerXYZ);
    test_rotation_matrix_to_euler_01!(rotation_matrix_to_euler_test_xzy_01, EulerXZY);
    test_rotation_matrix_to_euler_01!(rotation_matrix_to_euler_test_yxz_01, EulerYXZ);
    test_rotation_matrix_to_euler_01!(rotation_matrix_to_euler_test_yzx_01, EulerYZX);
    test_rotation_matrix_to_euler_01!(rotation_matrix_to_euler_test_zxy_01, EulerZXY);
    test_rotation_matrix_to_euler_01!(rotation_matrix_to_euler_test_zyx_01, EulerZYX);

    macro_rules! test_rotation_matrix_to_euler_02 {
        ($test_name:ident , $rotation_mode:ident) => {
            #[test]
            fn $test_name() {
                let res = rotation_matrix_to_euler(&glm::identity(), RotationModes::$rotation_mode);

                assert_vec_relative_eq!(res, glm::vec3(0.0, 0.0, 0.0), 1e-6);
            }
        };
    }

    test_rotation_matrix_to_euler_02!(rotation_matrix_to_euler_test_xyz_02, EulerXYZ);
    test_rotation_matrix_to_euler_02!(rotation_matrix_to_euler_test_xzy_02, EulerXZY);
    test_rotation_matrix_to_euler_02!(rotation_matrix_to_euler_test_yxz_02, EulerYXZ);
    test_rotation_matrix_to_euler_02!(rotation_matrix_to_euler_test_yzx_02, EulerYZX);
    test_rotation_matrix_to_euler_02!(rotation_matrix_to_euler_test_zxy_02, EulerZXY);
    test_rotation_matrix_to_euler_02!(rotation_matrix_to_euler_test_zyx_02, EulerZYX);
}
