use egui::emath::Numeric;
use enumflags2::BitFlags;

use super::super::bsdf::{SampleData, SamplingTypes, BSDF};
use super::super::intersectable::IntersectInfo;
use crate::math;
use crate::ui::DrawUI;
use crate::{glm, ui};

pub fn gtr2_aniso(ndot_h: f64, hdot_x: f64, hdot_y: f64, ax: f64, ay: f64) -> f64 {
    1.0 / (std::f64::consts::PI
        * ax
        * ay
        * ((hdot_x / ax).powf(2.0) + (hdot_y / ay).powf(2.0) + ndot_h.powf(2.0)).powf(2.0))
}

pub fn schlick_fresnel(u: f64) -> f64 {
    let m = u.clamp(0.0, 1.0);
    m.powf(5.0)
}

pub fn smith_ggx_aniso(ndot_v: f64, vdot_x: f64, vdot_y: f64, ax: f64, ay: f64) -> f64 {
    1.0 / (ndot_v + ((vdot_x * ax).powf(2.0) + (vdot_y * ay).powf(2.0) + ndot_v.powf(2.0)).sqrt())
}

pub struct Disney {
    color: glm::DVec4,
    specular: f64,
    metal: f64,
    spec_tint: f64,
    rough: f64,
    diffuse: f64,
    aniso: f64,
}

impl Disney {
    pub fn new(
        color: glm::DVec4,
        specular: f64,
        metal: f64,
        spec_tint: f64,
        rough: f64,
        diffuse: f64,
        aniso: f64,
    ) -> Self {
        Self {
            color,
            specular,
            metal,
            spec_tint,
            rough,
            diffuse,
            aniso,
        }
    }
}
impl BSDF for Disney {
    fn sample(
        &self,
        _wo: &glm::DVec3,
        intersect_info: &IntersectInfo,
        sampling_types: BitFlags<SamplingTypes>,
    ) -> Option<SampleData> {
        if sampling_types.contains(SamplingTypes::Reflection)
            && sampling_types.contains(SamplingTypes::Diffuse)
        {
            // Need to perform Uniform Random Sampling
            Some(SampleData::new(
                -(intersect_info.get_normal().unwrap() + math::random_in_unit_sphere()),
                SamplingTypes::Diffuse,
            ))
        } else {
            None
        }
    }

    fn eval(&self, wi: &glm::DVec3, wo: &glm::DVec3, intersect_info: &IntersectInfo) -> glm::DVec3 {
        let unit_vec: glm::DVec3 = glm::vec3(1.0, 1.0, 1.0);
        let color = glm::vec4_to_vec3(&self.color);
        let ndot_l = intersect_info.get_normal().unwrap().dot(wi);
        let ndot_v = intersect_info
            .get_normal()
            .unwrap()
            .dot(intersect_info.get_point());
        let half_vec = (wi + intersect_info.get_normal().unwrap()).normalize();
        let ndot_h = intersect_info.get_normal().unwrap().dot(&half_vec);
        let ldot_h = wi.dot(&half_vec);
        // Luminance Approximation
        let cdlin = glm::pow(&color, &glm::vec3(2.2, 2.2, 2.2));
        let lum_appr = 0.3 * cdlin[0] + 0.6 * cdlin[1] * 0.1 * cdlin[2];

        let ctint = if lum_appr > 0.0 {
            cdlin / lum_appr
        } else {
            glm::zero()
        };
        let cspec = glm::mix(
            &(self.specular * 0.08 * glm::mix(&unit_vec, &ctint, self.spec_tint)),
            &cdlin,
            self.metal,
        );
        // Evaluating the diffuse component
        // TODO: Decide if eval needs to be split into different passes or not
        let fd90minone = 2.0 * self.rough * ldot_h.powf(2.0) - 0.5;
        let fdl = 1.0 + (fd90minone * (1.0 - ndot_l).powf(5.0));
        let fdv = 1.0 + (fd90minone * (1.0 - ndot_v).powf(5.0));
        let diffuse_sum: f64 = self.diffuse * fdl * fdv * std::f64::consts::FRAC_1_PI * ndot_l;

        // Hanrahan-Kruger brdf
        let fss90 = ldot_h * ldot_h * self.rough;
        let fss = glm::mix_scalar(1.0, fss90, fdl);
        let ss = 1.25 * (fss * (1.0 / (ndot_l + ndot_v)));

        // Evaluating the specular part
        // TODO: Decide if this needs to be split into different passes or not
        let aspect = (1.0 - self.aniso * 0.9).sqrt();
        let ax = glm::max2_scalar(0.001, self.rough.powf(2.0) / aspect);
        let ay = glm::max2_scalar(0.001, self.rough.powf(2.0) * aspect);
        // TODO: Need to get tangent and bitangent at a point
        // let ds = gtr2_aniso(nodt_h, h.dot(intersect_info)
        let fh = schlick_fresnel(ldot_h);
        let fs = glm::mix(&cspec, &unit_vec, fh);

        color
    }

    fn get_bsdf_name(&self) -> &str {
        "Disney"
    }
}

impl DrawUI for Disney {
    fn draw_ui(&self, ui: &mut egui::Ui) {
        ui.label(format!("BSDF: {}", self.get_bsdf_name()));
    }

    fn draw_ui_mut(&mut self, ui: &mut egui::Ui) {
        ui::color_edit_button_dvec4(ui, "Base Color", &mut self.color);
    }
}
