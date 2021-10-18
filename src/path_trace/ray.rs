use crate::glm;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ray {
    origin: glm::DVec3,
    direction: glm::DVec3,
}

impl Ray {
    pub fn new(origin: glm::DVec3, direction: glm::DVec3) -> Self {
        Self { origin, direction }
    }

    pub fn get_origin(&self) -> &glm::DVec3 {
        &self.origin
    }

    pub fn get_direction(&self) -> &glm::DVec3 {
        &self.direction
    }

    pub fn at(&self, t: f64) -> glm::DVec3 {
        self.origin + t * self.direction
    }

    /// Checks if the ray intersects with the given triangle defined
    /// by the points `v1`, `v2`, and `v3`. Returns Some((distance to
    /// point of intersection, barycentric coords of the point of
    /// intersection)) when it intersects.
    ///
    /// Based on Blender's `isect_ray_tri_epsilon_v3()` defined in
    /// `math_geom.c`
    #[allow(clippy::many_single_char_names)]
    pub fn intersect_triangle(
        &self,
        v1: &glm::DVec3,
        v2: &glm::DVec3,
        v3: &glm::DVec3,
        epsilon: f64,
    ) -> Option<(f64, glm::DVec3)> {
        let e1 = v2 - v1;
        let e2 = v3 - v1;

        let p = glm::cross(self.get_direction(), &e2);
        let a = glm::dot(&e1, &p);
        if a == 0.0 {
            return None;
        }

        let f = 1.0 / a;
        let s = self.get_origin() - v1;

        let u = f * glm::dot(&s, &p);
        if u < -epsilon || u > 1.0 + epsilon {
            return None;
        }

        let q = glm::cross(&s, &e1);
        let v = f * glm::dot(self.get_direction(), &q);

        if v < -epsilon || (u + v) > 1.0 + epsilon {
            return None;
        }

        let lambda = f * glm::dot(&e2, &q);
        if lambda < 0.0 {
            return None;
        }

        Some((lambda, glm::vec3(1.0 - u - v, u, v)))
    }
}
