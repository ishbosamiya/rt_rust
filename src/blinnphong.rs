use crate::bsdf::BSDF;
use crate::glm;

pub struct BlinnPhong{}


impl BSDF for BlinnPhong{
    fn new() -> Self{
        BlinnPhong{}
    }
    fn sample(&self, 
        out : &glm::DVec3, 
        vertex : &glm::DVec3
    ) -> glm::DVec3 {
        return glm::zero();
    }

    fn eval(
        &self,
        l: &glm::DVec3,
        v: &glm::DVec3,
        n: &glm::DVec3,
        x: &glm::DVec3,
        y: &glm::DVec3,
    ) -> glm::DVec3{
        let divide_by_ndot_l: bool = true;
        
        let s = l+v;
        let h = s.normalize();
        let ndot_h = n.dot(&h);
        let ndot_l = n.dot(l);


        let mut val = ndot_h.powf(100.0_f64);

        if divide_by_ndot_l{
            val = val/ndot_l
        }

        return glm::vec3(val,val,val);

    }




}