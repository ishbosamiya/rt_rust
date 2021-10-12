use crate::glm;
use egui::emath::Numeric;
use rand::Rng;
extern crate rand;
use rand::thread_rng;
use crate::sampler;

pub fn microfacet_alpha_from_roughness(roughness : f64, 
    aniso : f64, 
    alpha_x : &mut f64, 
    alpha_y : &mut f64) {
        let sqr_rough = roughness.powf(2.0);

        if aniso >= 0.0_f64 {
            let aspect = (1.0_f64 - aniso * 0.9_f64).sqrt();
            *alpha_x = glm::max2_scalar(0.001_f64, sqr_rough / aspect);
            *alpha_y = glm::max2_scalar(0.001_f64, sqr_rough * aspect);
        }
        else {
            let aspect = (1.0_f64 + aniso * 0.9_f64).sqrt();
            *alpha_x = glm::max2_scalar(0.001_f64, sqr_rough * aspect);
            *alpha_y = glm::max2_scalar(0.001_f64, sqr_rough / aspect);
        }
}

pub fn sample_aniso_glossy(k : f64, s : f64) -> f64 {
    let hpi : f64 = glm::half_pi();
    if s < 0.25 {
        let b: f64 = (hpi * 4.0_f64 * s).tan();
        (k * b).atan()
    }
    else if s < 0.5 {
        let b: f64 = (hpi * (4.0_f64 * s - 1.0)).tan();
        (k * b).atan() + hpi
    }
    else if s < 0.75 {
        let b: f64 = (hpi * (4.0_f64 * s - 2.0)).tan();
        (k * b).atan() + std::f64::consts::PI
    }
    else {
        let b: f64 = (hpi * (4.0_f64 * s - 3.0)).tan();
        (k * b).atan() + std::f64::consts::PI + hpi
    }
}

// Here glossy and diffuse are reflectance values
// Unsure of exact variables in disney bsdf
pub fn sample_micro(outgoing : &glm::DVec3, 
    glossy : f64, 
    diffuse : f64, 
    aniso : f64,
    normal : &glm::DVec3,
    vertex : &glm::DVec3
) -> glm::DVec3 {

    let rd = glossy.clamp(0.0, 1.0);
    let rg = diffuse.clamp(0.0, 1.0);

    let avg_sum = rd + rg;
    
    assert!(avg_sum > 0.0);

    let m_pd = rd / avg_sum;
    let m_pg = 1.0_f64 - m_pd;

    let mut diffuse_weights = m_pd;
    let mut glossy_weights = m_pg;

    let total_weight = diffuse_weights + glossy_weights;
    assert!(total_weight > 0.0);
    let rcp_total = 1.0 / total_weight;
    diffuse_weights *= rcp_total;
    glossy_weights *= rcp_total;


    // Unsure of how to calculate shiny parameters
    // TODO Calc shiny params

    let mut rng = thread_rng();
    let x: f64 = rng.gen_range(0.0..1.0);
    let y: f64 = rng.gen_range(0.0..1.0);
    let z : f64 = rng.gen_range(0.0..1.0);

    let s = glm::vec3(x, y, z);
    let mut incoming = glm::zero();
    let mut h_vec = glm::zero();
    let exp : f64;

    if s[2] < diffuse_weights {
        let wi = sampler::cosine_hemisphere(&glm::vec2(s[0], s[1]));
        incoming = wi[0] * outgoing + wi[1] * normal + wi[2] * vertex;
    }
    else {
        // TODO Anisotropic calcs
        let phi : f64 = sample_aniso_glossy(aniso, s[0]);
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();

        // Unsure of formula check once more
        exp = (glossy * cos_phi.powf(2.0)) + (aniso * sin_phi.powf(2.0));

        let cos_theta = (1.0_f64 - s[1]).powf(1.0 / (exp + 1.0));
        let sin_theta = (1.0_f64 - cos_theta.powf(2.0)).sqrt();

        let trig_vec = glm::vec3(cos_phi * sin_theta, cos_theta, sin_phi * sin_theta);
        h_vec = trig_vec[0] * outgoing + trig_vec[1] * normal + trig_vec[2] * vertex;
        incoming = (outgoing - 2.0_f64 * outgoing.dot(&h_vec) * h_vec).normalize();
    }
    h_vec = (incoming + outgoing).normalize();

    incoming
}