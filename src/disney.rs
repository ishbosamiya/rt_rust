use crate::bsdf::BSDF;
use crate::glm;
use crate::bsdfutils::Utils;


// File for main disney brdf code
pub struct Disney {
    pub basecolor: glm::DVec3,
    pub metallic: f64,
    pub subsurface: f64,
    pub specular: f64,
    pub roughness: f64,
    pub specular_tint: f64,
    pub anisotropic: f64,
    pub sheen: f64,
    pub sheen_tint: f64,
    pub clearcoat: f64,
    pub clearcoat_glass: f64
}


impl BSDF for Disney {
    fn new() -> Self {
        Disney {basecolor: glm::DVec3::new(0.82, 0.67, 0.16),
            metallic: 0.0_f64, 
            subsurface: 0.0_f64, 
            specular: 0.5_f64, 
            roughness: 0.5_f64, 
            specular_tint: 0.0_f64, 
            anisotropic: 0.0_f64, 
            sheen: 0.0_f64, 
            sheen_tint: 0.5_f64,
            clearcoat: 0.0_f64,
            clearcoat_glass: 1.0_f64
        }
    }
    
    fn eval(&self,
        l: &glm::DVec3,
        v: &glm::DVec3,
        n: &glm::DVec3,
        x: &glm::DVec3,
        y: &glm::DVec3,
    ) -> glm::DVec3 {
        // Enter main eval code
        let ndot_l = n.dot(l);
        let ndot_v = n.dot(v);
        
        let pi = 3.14159265358979323846;

        if ndot_l < 0.0 || ndot_v < 0.0 {
            return glm::DVec3::new(0.0, 0.0, 0.0);
        }
        //let mut h : glm::DVec3;
        let h = (l + v).normalize();

        let ndot_h = n.dot(&h);
        let ldot_h = l.dot(&h);

        let hdot_x = h.dot(x);

        let hdot_y = h.dot(y);
        // Utility structure
        let util: Utils = Utils::new();
        // Calculate colour if required here
        let cdlin = util.mon2lin(&self.basecolor);
        // Calculate lumincance approx
        let cdlum = 0.3_f64 * cdlin.x + 0.6_f64 * cdlin.y + 0.1_f64 * cdlin.z;

        let newvec = glm::DVec3::new(cdlin.x / cdlum, cdlin.y / cdlum, cdlin.z / cdlum);
        let ctint: glm::DVec3;
        ctint = if cdlum > 0.0_f64 {newvec} else {glm::DVec3::new(1.0, 1.0, 1.0)};
        
        let cspec0: glm::DVec3;
        // TODO Check this function
        let spec_vec: glm::DVec3 = self.specular * 0.8_f64 * util.mix(&glm::DVec3::new(1.0_f64, 1.0_f64, 1.0_f64),&ctint,self.specular_tint);
        cspec0 = util.mix(&spec_vec, &cdlin,self.metallic);

        let csheen: glm::DVec3;
        csheen = util.mix(&glm::DVec3::new(1.0,1.0,1.0), &ctint, self.sheen_tint);

        let fl = util.schlick_fresnel(ndot_l);

        let fv = util.schlick_fresnel(ndot_v);

        let fd90 = 0.5_f64 + 2.0_f64 * ldot_h*ldot_h * self.roughness;

        let fd = util.mixnum(1.0_f64, fd90,fl)* util.mixnum(1.0,fd90,fv);

        let fss90 = ldot_h * ldot_h * self.roughness;

        let fss = util.mixnum(1.0_f64, fss90, fl) * util.mixnum(1.0_f64, fss90, fv);

        let ss = 1.25 * (fss* (1.0_f64/(ndot_l+ndot_v) - 0.5_f64)+ 0.5_f64);
        
        
        // Specular Part
        let aspect = (1.0_f64-self.anisotropic*0.9_f64).sqrt();
        let ax = 0.001_f64.max(self.roughness.sqrt()/aspect);
        let ay = 0.001_f64.max(self.roughness.sqrt()*aspect);
        let ds = util.gtr2_aniso(ndot_h, hdot_x, hdot_y, ax, ay);
        let fh = util.schlick_fresnel(ldot_h);

        let fs: glm::DVec3;

        fs = util.mix(&cspec0, &glm::DVec3::new(1.0,1.0,1.0), fh);

        let mut gs = util.smithg_ggx_aniso(ndot_l,l.dot(x),l.dot(y),ax,ay);

        gs *= util.smithg_ggx_aniso(ndot_v,v.dot(x),v.dot(y),ax,ay);

        let fsheen: glm::DVec3;
        fsheen = fh * self.sheen * csheen;

        let dr = util.gtr1(ndot_h, util.mixnum(0.1_f64, 0.001_f64, self.clearcoat_glass));

        let fr = util.mixnum(0.4_f64, 1.0_f64, fh);

        let gr = util.smithg_ggx(ndot_l, 0.25_f64) * util.smithg_ggx(ndot_v, 0.25_f64);

        // Check main code reference to fix error
        // Unsure of main code
        let clear_val = 0.25_f64 * self.clearcoat * gr * fr * dr;
        let clear_vec = glm::DVec3::new(clear_val, clear_val, clear_val);
        return ((1.0_f64 / pi) * util.mixnum(fd, ss, self.subsurface, ) * cdlin + fsheen) * (1.0_f64 - self.metallic) + gs * fs * ds + clear_vec;
    }
}