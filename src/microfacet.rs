use crate::glm;
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

// Here glossy and diffuse are reflectance values
// Unsure of exact variables in disney bsdf
pub fn sample_micro(outgoing : &glm::DVec3, 
    glossy : f64, 
    diffuse : f64, 
    shiny : f64,
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
        h_vec = (incoming + outgoing).normalize();
    }
    else {
        // TODO Anisotropic calcs
    }

    glm::zero()
}