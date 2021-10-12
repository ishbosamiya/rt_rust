use crate::disney::Disney;
use crate::glm;

// Removed front and back facing as it is already present in intersect info

// Structure for the Template
pub struct BSDFTemplate {
    pub roughness: f64,
    pub brightness: f64,
    pub opacity: f64
}

// Implementing the template with various functions
// Need to finish this function as provided in brdftemplatesphere.frag
// Need not put color as of now
impl BSDFTemplate {
    fn compute_with_directional_light(
        &self,
        _surf: &glm::DVec3,
        l: &glm::DVec3,
        view: &glm::DVec3,
        n: &glm::DVec3,
        _x: &glm::DVec3,
        _y: &glm::DVec3,
    ) -> glm::DVec3 {
        // let zerovec = glm::zero();
        let disney_model: Disney = BSDF::new();
        let mut b = disney_model.eval(l, view, n, _x, _y);
        // let mut b = s;
        b = glm::max2(&b, &glm::zero());

        b = b * n.dot(l);

        return b;
    }
    // Finish this function from main of above file
    pub fn setup(&self, ray: &glm::DVec3, vertex: &glm::DVec3) -> glm::DVec3 {
        // Replace ray with vertex vector in world space
        // Ray then becomes incident light vector
        let normal = vertex.normalize();
        let tangent = (glm::vec3(0.0_f64, 1.0_f64, 0.0_f64).cross(&normal)).normalize();
        let bitangent = (normal.cross(&tangent)).normalize();

        let surfacepos: glm::DVec3 = vertex.normalize();

        let viewvec = glm::vec3(0.0_f64, 0.0_f64, 1.0_f64);
        let mut b = self.compute_with_directional_light(
            &surfacepos,
            ray,
            &viewvec,
            &normal,
            &tangent,
            &bitangent,
        );

        b = b * self.brightness;

        // Calculate exposure - TBD
        // b = b * self.opacity.powf(2.0)
        // Check if gamma is roughness or not
        let gamma_factor = 1.0_f64 / self.roughness;
        let gamma_vec = glm::vec3(gamma_factor, gamma_factor, gamma_factor);

        b = glm::pow(&b, &gamma_vec);

        // Check this once more
        // let maxvec = glm::vec3(1.0, 1.0, 1.0);
        // let frag_color = glm::vec4(glm::clamp_vec(&b, &glm::zero(), &maxvec), 1.0);

        return b;
    }
}

// Main trait for implementing the BSDF
pub trait BSDF {
    fn new() -> Self;
    // TODO Implement Sample struct if needed (may not need for lambert)

    



    fn sample(&self, 
        out : &glm::DVec3, 
        vertex : &glm::DVec3
    ) -> glm::DVec3;
    fn eval(
        &self,
        l: &glm::DVec3,
        v: &glm::DVec3,
        n: &glm::DVec3,
        x: &glm::DVec3,
        y: &glm::DVec3,
    ) -> glm::DVec3;
}
