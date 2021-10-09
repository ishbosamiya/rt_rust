use crate::blinn::Blinn;
use crate::blinnphong::BlinnPhong;
use crate::disney::Disney;
use crate::glm;

// Removed front and back facing as it is already present in intersect info

// Structure for the Template
pub struct BSDFTemplate {
    pub roughness: f64,
    pub brightness: f64,
    pub opacity: f64,
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
        let blinn_model: BlinnPhong = BSDF::new();
        let s = blinn_model.eval(l, view, n, _x, _y);
        let mut b = s;

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

        b = b * self.brightness  ;

        // Calculate exposure - TBD
        // b = b * self.opacity.powf(2.0)
        // Check if gamma is roughness or not

        let invgamma = 1.0_f64 / self.roughness;

        let new = glm::vec3(b.x.powf(invgamma), b.y.powf(invgamma), b.z.powf(invgamma));

        return new;
    }
}

// Main trait for implementing the BSDF
pub trait BSDF {
    fn new() -> Self;
    // fn sample(event : &SubsurfaceScatterEvent) -> bool
    fn eval(
        &self,
        l: &glm::DVec3,
        v: &glm::DVec3,
        n: &glm::DVec3,
        x: &glm::DVec3,
        y: &glm::DVec3,
    ) -> glm::DVec3;
}
