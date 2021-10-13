use crate::glm;

/// File for the Sampler
/// Contains Uniform path sampler and sample warp
pub fn cosine_hemisphere(xi: &glm::DVec2) -> glm::DVec3 {
    let phi = xi[0] * 2.0_f64 * std::f64::consts::PI;
    let r = (xi[1]).sqrt();

    glm::vec3(
        phi.cos() * r,
        phi.sin() * r,
        (glm::max2_scalar(1.0_f64 - xi.y, 0.0_f64)).sqrt(),
    )
}

pub fn uniform_hemisphere(xi: &glm::DVec2) -> glm::DVec3 {
    let phi = (2.0_f64 * std::f64::consts::PI) * xi[0];
    let r = (glm::max2_scalar(1.0 - xi[1] * xi[1], 0.0)).sqrt();
    glm::vec3(phi.cos() * r, phi.sin() * r, xi[1])
}

/// May need to move to a different file
pub struct Sampler {}

impl Sampler {
    /*
    /// TBD Need an outstream module
    fn saveState(out : &outstream) {
        /// Write to file
    }
    /// PCG Random number generator
    fn next() -> u64 {
        let oldState = self.state;
        self.state = oldState * 6364136223846793005 + (self.sequence | 1);
        let mut xorshifted32 : u32 = ((oldState >> 18) ^ oldState) >> 27;
        let mut rot : u32 = oldState >> 59;
        return (xorshifted32 >> rot) | (xorshifted32 << (rot & 31))
    }

    pub fn nextBoolean(p : &f64) -> bool {
        let mut i : u64 = self.next();
        let mut k : f64 = (i >> 9_u64) | 0x3F800000_u64) - 1.0;
        return k < p;
    }

    pub fn invertPhi(w : &Vec3, mu : &f64) -> f64 {
        let INV_TWO_PI = 0.5_f64 * (1.0_f64 / 3.1415926536_f64)
        let result = if (w.x == 0.0 && w.y == 0.0) {mu*INV_TWO_PI} else {atan2(w.y, w.x) * INV_TWO_PI};
        if (result < Vec3::new(0.0, 0.0, 0.0))
            result += Vec3::new(1.0, 1.0, 1.0);
        return result;
    }

    pub fn uniformHemispherePdf(p : &Vec3) -> f64 {
        let INV_TWO_PI = 0.5_f64 * (1.0_f64 / 3.1415926536_f64);
        return INV_TWO_PI;
    }
    */

    /*
    pub fn cosineHemispherePdf(p : &Vec3) -> f64 {
        let INV_PI = (1.0_f64 / 3.1415926536_f64)
        return abs(p.z) * INV_PI;
    }

    pub fn invertCosineHemisphere(w : &Vec3, mu : &f64) -> Vec2
    {
        return glm::vec2::new(invertPhi(w, mu), max(1.0 - w.z * w.z, 0.0));
    }

    pub fn uniformDisk(xi : &Vec2) -> Vec3 {
        let mut phi = xi.x * 2.0 * 3.1415926536;
        let r = sqrt(xi.y);
        return Vec3::new(std::cos(phi)*r, std::sin(phi)*r, 0.0);
    }

    pub fn uniformDiskPdf() -> f64 {
        return (1.0_f64 / 3.1415926536_f64);
    }

    // TBD COMPLETE THIS FUNCTION
    // pub fn invertUniformDisk(p : &Vec3, mu : &f64) -> Vec2 {
    //     return glm::vec2::new(invertPhi(p, mu), p.x*p.x + p.y*p.y);
    // }

    pub fn uniformCylinder(xi : &Vec2) -> Vec3 {
        let TWO_PI =  2.0 * 3.1415926536;
        let mut phi = xi.x * TWO_PI;
        return Vec3::new(
            std::cos(phi),
            std::sin(phi),
            xi.y * 2.0 - 1.0
        );
    }

    pub fn uniformCylinderPdf() -> f64 {
        return (1.0_f64 / 3.1415926536_f64);
    }

    pub fn uniformSphere(xi : &Vec2) -> Vec3 {
        let TWO_PI = 2.0 * 3.1415926536;
        let phi = xi.x * TWO_PI;
        let mut z = xi.y * 2.0 - 1.0;
        let r = sqrt(max(1.0 - z*z, 0.0));

        return Vec3::new(
            std::cos(phi)*r,
            std::sin(phi)*r,
            z
        );
    }

    pub fn uniformSpherePdf() -> f64 {
        return 0.25_f64 * (1.0_f64 / 3.1415926536_f64);
    }

    pub fn invertUniformSphere(w : &Vec3, mu : &f64) -> Vec2
    {
        return glm::vec2::new(invertPhi(w, mu), (w.z + 1.0)*0.5);
    }

    pub fn uniformSphericalCap(xi : &Vec2, cosThetaMax : &f64) -> Vec3 {
        let TWO_PI = 2.0 * 3.1415926536;
        let phi = xi.x * TWO_PI;
        let mut z = xi.y * (1.0 - cosThetaMax) + cosThetaMax;
        let r = sqrt(max(1.0 - z*z, 0.0));
        return Vec3::new(
            cos(phi)*r,
            sin(phi)*r,
            z
        );
    }

    pub fn uniformSphericalCapPdf(cosThetaMax : &f64) -> f64 {
        let INV_TWO_PI = 0.5_f64 * (1.0_f64 / 3.1415926536_f64);
        return INV_TWO_PI/(1.0f - cosThetaMax);
    }

    pub fn invertUniformSphericalCap(w : &Vec3, cosThetaMax : &f64, xi : &Vec2, mu : &f64) -> bool
    {
        let mut xiY = (w.z - cosThetaMax)/(1.0 - cosThetaMax);
        if (xiY >= 1.0 || xiY < 0.0f)
            return false;

        xi = glm::vec2::new(invertPhi(w, mu), xiY);
        return true;
    }

    pub fn phongHemisphere(xi : &Vec2, n : &f64) -> Vec3
    {
        let phi = xi.x * 2.0 * 3.1415926536;
        let mut cosTheta = xi.y.pow(1.0/(n + 1.0));
        let r = std::sqrt(max(1.0 - cosTheta*cosTheta, 0.0));
        return Vec3::new(std::cos(phi)*r, std::sin(phi)*r, cosTheta);
    }

    pub fn phongHemispherePdf(v : &Vec3, n : &f64) -> f64 {
        let INV_TWO_PI = 0.5_f64 * (1.0_f64 / 3.1415926536_f64);
        return INV_TWO_PI*(n + 1.0)*v.z.pow(n);
    }

    pub fn invertPhongHemisphere(w : &Vec3, n : &f64, mu : &f64) -> Vec2 {
        return Vec2::new(invertPhi(w, mu), w.z.pow(n + 1.0));
    }

    pub fn uniformTriangleUv(xi : &Vec2) -> Vec2 {
        let mut uSqrt = xi.x.sqrt();
        let alpha = 1.0 - uSqrt;
        let beta = (1.0 - xi.y)*uSqrt;

        return glm::vec2::new(alpha, beta);
    }

    pub fn uniformTriangle(xi : &Vec2, a : &Vec3, b : &Vec3, c : &Vec3) -> Vec3 {
        let mut uv : Vec2 = uniformTriangleUv(xi);
        return a*uv.x + b*uv.y + c*(1.0 - uv.x - uv.y);
    }

    pub fn powerHeuristic(pdf0 : &f64, pdf1 : &f64) -> f64 {
        return (pdf0*pdf0)/(pdf0*pdf0 + pdf1*pdf1);
    }

    /// TODO Combine box code with below
    pub fn projectedBox(box : &Box, direction : &Vec3, faceXi : &f64, xi : &Vec2) -> Vec3 {
        let mut diag: Box = box.diagonal();
        let areaX = diag.y*diag.z*std::abs(direction.x);
        let areaY = diag.z*diag.x*std::abs(direction.y);
        let areaZ = diag.x*diag.y*std::abs(direction.z);

        let u : f64 = faceXi * (areaX + areaY + areaZ);

        let mut result : Vec3::new(0.0, 0.0, 0.0);
        if (u < areaX) {
            result.x = box.max.x if direction.x < 0.0 else box.min.x;
            result.y = box.min().y + diag.y * xi.x;
            result.z = box.min().z + diag.z * xi.y;
        } else if (u < areaX + areaY) {
            result.y = box.max().y if direction.y < 0.0 else box.min().y;
            result.z = box.min().z + diag.z * xi.x;
            result.x = box.min().x + diag.x * xi.y;
        } else {
            result.z = box.max().z if direction.z < 0.0 else box.min().z;
            result.x = box.min().x + diag.x * xi.x;
            result.y = box.min().y + diag.y * xi.y;
        }

        return result;
    }

    pub fn projectedBoxPdf(box : &Box, direction : &Vec3) -> f64 {
        let mut diag : Vec3 = box.diagonal();
        let areaX = diag.y * diag.z * direction.x.abs();
        let areaY = diag.z * diag.x * direction.y.abs();
        let areaZ = diag.x * diag.y * direction.z.abs();

        return 1.0 / (areaX + areaY + areaZ);
    }

    pub fn invertProjectedBox(box : &Box, o : &Vec3, d : &Vec3, faceXi : &f64, xi : &Vec2, mu : &f64) -> bool {
        let invD: Vec3 = 1.0/d;
        let mut relMin: Vec3::new((box.min() - o),(box.min() - o),(box.min() - o));
        let mut relMax: Vec3::new((box.max() - o),(box.max() - o),(box.max() - o));

        let ttMin : f64 = 0;
        let ttMax : f64 = 1e30_f64;
        let dim = -1;
        for i in 0..3) {
            if (invD[i] >= 0.0f) {
                ttMin = max(ttMin, relMin[i]*invD[i]);
                let mut x: f64 = relMax[i]*invD[i];
                if (x < ttMax) {
                    ttMax = x;
                    dim = i;
                }
            } else {
                let mut x: f64 = relMin[i]*invD[i];
                if (x < ttMax) {
                    ttMax = x;
                    dim = i;
                }
                ttMin = max(ttMin, relMax[i]*invD[i]);
            }
        }

        if (ttMin <= ttMax) {
            let mut diag : Vec3 = box.diagonal();
            let dim1 = (dim + 1) % 3;
            let dim2 = (dim + 2) % 3;

            xi = Vec2::new(
                (o[dim1] + d[dim1]*ttMax - box.min()[dim1])/diag[dim1],
                (o[dim2] + d[dim2]*ttMax - box.min()[dim2])/diag[dim2]
            );

            let mut areaX = diag.y * diag.z * d.x.abs();
            let mut areaY = diag.z * diag.x * d.y.abs();
            let mut areaZ = diag.x * diag.y * d.z.abs();

            if (dim == 0)
                faceXi = mu*areaX/(areaX + areaY + areaZ);
            else if (dim == 1)
                faceXi = (areaX + mu*areaY)/(areaX + areaY + areaZ);
            else
                faceXi = (areaX + areaY + mu*areaZ)/(areaX + areaY + areaZ);


            return true;
        }
        return false;
    }
    */
}
