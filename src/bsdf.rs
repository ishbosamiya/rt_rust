use crate::glm;
use crate::intersectable::IntersectInfo;

pub trait BSDF {
    /// Calculates `wi` given `wo`.
    ///
    /// `wo`: outgoing ray direction
    /// `wi`: incoming ray direction
    /// `intersect_info`: information at the point of intersection
    ///
    /// Need to calculate the incoming ray direction since in ray
    /// tracing, we are moving from the camera into the scene, not
    /// from the light sources towards the camera. So it is reversed,
    /// we have the outgoing ray but don't have the incoming ray.
    ///
    /// TODO: need to send sampling type(s) as well (diffuse or glossy
    /// or reflection) when max number bounces for each of these types
    /// is implemented.
    fn sample(&self, wo: &glm::DVec3, intersect_info: &IntersectInfo) -> glm::DVec3;

    /// Calculates the colour/intensity of light that moves from `wi` towards `wo`.
    ///
    /// `wo`: outgoing ray direction
    /// `wi`: incoming ray direction
    /// `intersect_info`: information at the point of intersection
    ///
    /// TODO: when different sampling type(s) are used, instead of
    /// just returning the colour/intensity of light, it will need to
    /// evaluate and update the value for each pass (diffuse, glossy,
    /// reflection).
    fn eval(&self, wi: &glm::DVec3, wo: &glm::DVec3, intersect_info: &IntersectInfo) -> glm::DVec3;
}
