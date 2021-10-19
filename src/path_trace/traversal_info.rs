use crate::glm;

use super::ray::Ray;

pub struct SingleRayInfo {
    /// the ray for which the info is defined
    ray: Ray,
    /// the point of intersection for the ray
    ///
    /// it is possible for the ray to be shaded by the environment so
    /// `co` may not exist, thus is made optional
    co: Option<glm::DVec3>,
    /// color/intensity of light of the ray
    color: glm::DVec3,
}

impl SingleRayInfo {
    /// Creates new `SingleRayInfo`
    pub fn new(ray: Ray, co: Option<glm::DVec3>, color: glm::DVec3) -> Self {
        Self { ray, co, color }
    }

    /// Get a reference to the single ray info's color.
    pub fn get_ray(&self) -> &Ray {
        &self.ray
    }

    /// Get a reference to the single ray info's co.
    pub fn get_co(&self) -> &Option<glm::DVec3> {
        &self.co
    }

    /// Get a reference to the single ray info's color.
    pub fn get_color(&self) -> &glm::DVec3 {
        &self.color
    }
}

pub struct TraversalInfo {
    traversal: Vec<SingleRayInfo>,
}

impl TraversalInfo {
    pub fn new() -> Self {
        Self {
            traversal: Vec::new(),
        }
    }

    pub fn get_traversal(&self) -> &[SingleRayInfo] {
        self.traversal.as_slice()
    }

    pub fn add_ray(&mut self, info: SingleRayInfo) {
        // TODO(ish): add some assertions to ensure that the traversal
        // path can form a continuous path
        self.traversal.push(info);
    }
}

impl Default for TraversalInfo {
    fn default() -> Self {
        Self::new()
    }
}
