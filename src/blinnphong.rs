use crate::bsdf::BSDF;
use crate::glm;

pub struct BlinnPhong {}

impl BSDF for BlinnPhong {
    fn new() -> Self {
        BlinnPhong {}
    }

    fn sample(&self, _out: &glm::DVec3, _vertex: &glm::DVec3) -> glm::DVec3 {
        todo!("BlinnPhong sample still needs to be calculated")
    }

    fn eval(
        &self,
        incident_vector: &glm::DVec3,
        view_vector: &glm::DVec3,
        normal_vector: &glm::DVec3,
        _tangent: &glm::DVec3,
        _bitangent: &glm::DVec3,
    ) -> glm::DVec3 {
        // l: incident_vector
        // v: view_vector
        // n: normal_vector
        // x: tangent
        // y: bitangent
        // h: incident_plus_view_normalized

        let divide_by_ndot_l = true;

        let h = (incident_vector + view_vector).normalize();
        let ndot_h = normal_vector.dot(&h);
        let ndot_l = normal_vector.dot(incident_vector);

        let mut val = ndot_h.powf(100.0_f64);

        if divide_by_ndot_l {
            val /= ndot_l
        }

        glm::vec3(val, val, val)
    }
}
