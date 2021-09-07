use nalgebra_glm as glm
use std::cmp
use crate::math::{Scalar,Vec3,saturate};
use crate::ray::Ray;
use crate::bsdf::{Material, BSDFData, GeomData};
use crate::subsurfacescatter::{SubsurfaceScatterEvent};


/// Function for calculating the fresnel dielectric 
pub fn fresnel_dielectric(cosi : f64, eta : f64) -> f64 {
    let c = cosi.abs();
    let mut g = eta * eta - 1 + c * c;
    if (g > 0) {
        g = g.sqrt();
        let a = (g - c) / (g + c);
        let  b = (c * (g + c) - 1) / (c * (g - c) + 1);

        return 0.5 * a * a * (1 + b * b);
    } 

    return 1.0;
}

impl BSDFData for Diffuse {
    fn sample(event : &SubsurfaceScatterEvent) -> bool {
        bool sampleR = event.requestedLobe.test(BsdfLobes::DiffuseReflectionLobe);
        bool sampleT = event.requestedLobe.test(BsdfLobes::DiffuseTransmissionLobe);
        if (!sampleR && !sampleT)
            return False;

        let transmittanceProbability = sampleR && sampleT ? _transmittance : (sampleR ? 0.0_f64 : 1.0_f64);
        let transmit = event.sampler.nextBoolean(transmittanceProbability);
        let mut weight :f64 = sampleR && sampleT ? 1.0_f64 : (transmit ? _transmittance : 1.0_f64 - _transmittance);

        event.wo = SampleWarp::cosineHemisphere(event.sampler.next2D());
        event.wo.z = event.wo.z * event.wi.z;
        if (transmit)
            event.wo.z = -event.wo.z;
        event.pdf = SampleWarp::cosineHemispherePdf(event.wo);
        event.weight = albedo(event.info) * weight;
        event.sampledLobe = BsdfLobes::DiffuseTransmissionLobe;
        return True;
    }

    ///pub fn invert(samplet : &PathSampleGenerator) -> bool;

    pub fn eval(event : &SubsurfaceScatterEvent) -> Vec3 {
        if (!event.requestedLobe.test(BsdfLobes::DiffuseTransmissionLobe))
        return Vec3::new(0.0, 0.0, 0.0);

        let mut factor : f64 = event.wi.z * event.wo.z < 0.0_f64 ? _transmittance : 1.0_f64 - _transmittance;
        return albedo(event.info) * factor * (1/3.14) * abs(event.wo.z);    /// Check formula and if 1/PI is correct
    }

    pub fn pdf(event : &SubsurfaceScatterEvent) -> f64 {
        /// TODO Write the test function for the different lobes
        let sampleR = event.requestedLobe.test(BsdfLobes::DiffuseReflectionLobe);
        let sampleT = event.requestedLobe.test(BsdfLobes::DiffuseTransmissionLobe);
        if (!sampleR && !sampleT)
            return 0.0_f64;

        let mut transmittanceProbability = sampleR && sampleT ? _transmittance : (sampleR ? 0.0_f64 : 1.0_f64);

        let mut factor = event.wi.z * event.wo.z < 0.0_f64 ? transmittanceProbability : 1.0_f64 - transmittanceProbability;
        return factor * SampleWarp::cosineHemispherePdf(event.wo);
    }
}