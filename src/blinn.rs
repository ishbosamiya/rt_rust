use crate::math::{Vec3};
use crate::bsdf::{BSDF};

fn normalize(T : &Vec3) -> Vec3 {
    /// TODO Finish this function
    let mut sumofsq : f64 = T.x.pow(2) + T.y.pow(2) + T.z.pow(2);
    let v = Math.sqrt(sumofsq);
    let mut N : Vec3 = T / v;
    return N;
}

impl BSDF {
    fn eval(L : &Vec3, V : &Vec3, N : &Vec3, X : &Vec3, Y : &Vec3) -> Vec3 {
        let include_fresnel : bool = True;
        let divide_by_NdotL : bool = True;
        let H : Vec3 = normalize(L + V);

        let NdotH = N.dot(H);
        let VdotH = V.dot(H);
        let NdotL = N.dot(L);
        let NdotV = N.dot(V);

        let mut x = Math.acos(NdotH) * 100;
        let D = Math.exp( -x * x);
        let mut G : f64;
        if (NdotV < NdotL) {
            G = if ((2 * NdotV * NdotH < VdotH)) {2 * NdotH / VdotH} else {1.0 / NdotV};
        }
        else {
            G = if ((2 * NdotL * NdotH < VdotH)) {2 * NdotH * NdotL / (VdotH * NdotV)} else {1.0 / NdotV};
        }
        let c = VdotH;
        let g = Math.sqrt(2.5 * 2.5 + c * c - 1);
        let mut F : f64 = 0.0;
        /// Double Check this value
        F = 0.5 * (g - c).pow(2) / (g + c).pow(2) * (1 + (c * (g + c) - 1).pow(2)) / (c * (g - c) + 1).pow(2);

        let mut val : f64 = 0.0;
        if (NdotH < 0.0) {
            val = 0.0;
        }
        else {
            let fresnel = if (include_fresnel) {F} else {1.0};
            val = D * G * fresnel;
        }

        if (divide_by_NdotL) {
            val = Some(val / N.dot(L));
        }
        return Vec3::new(val, val, val);

    }
}