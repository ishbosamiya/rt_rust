use itertools::Itertools;
use lazy_static::lazy_static;
use nalgebra::RealField;

use crate::glm;

/// TODO:
/// RGB to CIE XYZ,
/// CIE XYZ to RGB,
/// CIE XYZ to Spectrum,
/// Spectrum to CIE XYZ,
/// More operators for Spectrum,
///
/// # References
///
/// https://en.wikipedia.org/wiki/Illuminant_D65

/// A single wavelength with it's corresponding intensity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample<T> {
    wavelength: T,
    intensity: T,
}

impl<T> Sample<T> {
    pub fn new(wavelength: T, intensity: T) -> Self {
        Self {
            wavelength,
            intensity,
        }
    }

    /// Get a reference to the sample's wavelength.
    pub fn get_wavelength(&self) -> &T {
        &self.wavelength
    }

    /// Get a reference to the sample's intensity.
    pub fn get_intensity(&self) -> &T {
        &self.intensity
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Sample<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.get_wavelength(), self.get_intensity())
    }
}

/// A generic spectrum of any number of wavelengths and of any type
/// `T`. Stores the wavelengths that define the spectrum and their
/// corresponding intensities.
#[derive(Debug, Clone)]
pub struct TSpectrum<T> {
    /// Samples of the spectrum, always in ascending order of the wavelengths.
    ///
    /// Maintaining ordering of the wavelengths allows for certain
    /// optimizations. For example, when adding two spectrums
    /// together, a sort of merging of the samples can be done (think
    /// of merge sort) as opposed to finding the appropriate
    /// wavelength through linear search for each of the spectrums.
    samples: Vec<Sample<T>>,
}

impl<T> TSpectrum<T> {
    pub fn new(samples: Vec<Sample<T>>) -> Self {
        Self { samples }
    }

    /// Returns the number of samples in the spectrum.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Returns true if the spectrum has no samples
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Returns the samples of the spectrum
    pub fn get_samples(&self) -> &[Sample<T>] {
        &self.samples
    }
}

impl<T: RealField> TSpectrum<T> {
    /// Generate a spectrum with wavelengths of the given range and
    /// step size and their intensities as 0
    ///
    /// Current limitation is that the wavelengths have to be
    /// `usize`. [`std::ops::RangeInclusive`]<T> would require that
    /// the [`std::iter::Step`] be implemented for `T` which is
    /// nightly only :(
    pub fn zeros(wavelength_range: std::ops::RangeInclusive<usize>, step: usize) -> Self {
        Self::new(
            wavelength_range
                .step_by(step)
                .map(|wavelength| Sample::new(T::from_usize(wavelength).unwrap(), T::zero()))
                .collect(),
        )
    }

    /// Generate a spectrum with wavelengths of the given range and
    /// step size and their intensities as 1.0
    ///
    /// Current limitation is that the wavelengths have to be
    /// `usize`. [`std::ops::RangeInclusive`]<T> would require that
    /// the [`std::iter::Step`] be implemented for `T` which is
    /// nightly only :(
    pub fn ones(wavelength_range: std::ops::RangeInclusive<usize>, step: usize) -> Self {
        Self::new(
            wavelength_range
                .step_by(step)
                .map(|wavelength| Sample::new(T::from_usize(wavelength).unwrap(), T::one()))
                .collect(),
        )
    }

    /// Convert sRGB to Spectrum.
    ///
    /// Reference: <https://graphics.geometrian.com/research/spectral-primaries.html>
    pub fn from_srgb(srgb: &glm::TVec3<T>) -> Self {
        &SRGB_R_SPECTRUM * srgb[0] + &SRGB_G_SPECTRUM * srgb[1] + &SRGB_B_SPECTRUM * srgb[2]
    }
}

pub type Spectrum = TSpectrum<f32>;
pub type DSpectrum = TSpectrum<f64>;

macro_rules! spectrum_add {
    ( $lhs:ty, $rhs:ty ) => {
        impl<T: RealField> std::ops::Add<$rhs> for $lhs {
            type Output = TSpectrum<T>;

            fn add(self, rhs: $rhs) -> Self::Output {
                let lhs_len = self.samples.len();
                let rhs_len = rhs.samples.len();
                let mut samples = Vec::with_capacity(lhs_len.max(rhs_len));
                let mut lhs_iter = 0;
                let mut rhs_iter = 0;

                while lhs_iter < lhs_len && rhs_iter < rhs_len {
                    let lhs_sample = &self.samples[lhs_iter];
                    let rhs_sample = &rhs.samples[rhs_iter];
                    if lhs_sample.wavelength < rhs_sample.wavelength {
                        samples.push(Sample::new(lhs_sample.wavelength, lhs_sample.intensity));
                        lhs_iter += 1;
                    } else if lhs_sample.wavelength == rhs_sample.wavelength {
                        samples.push(Sample::new(
                            lhs_sample.wavelength,
                            lhs_sample.intensity + rhs_sample.intensity,
                        ));
                        lhs_iter += 1;
                        rhs_iter += 1;
                    } else {
                        samples.push(Sample::new(rhs_sample.wavelength, rhs_sample.intensity));
                        rhs_iter += 1;
                    }
                }

                while lhs_iter < lhs_len {
                    let lhs_sample = &self.samples[lhs_iter];
                    samples.push(Sample::new(lhs_sample.wavelength, lhs_sample.intensity));
                    lhs_iter += 1;
                }

                while rhs_iter < rhs_len {
                    let rhs_sample = &rhs.samples[rhs_iter];
                    samples.push(Sample::new(rhs_sample.wavelength, rhs_sample.intensity));
                    rhs_iter += 1;
                }

                Self::Output::new(samples)
            }
        }
    };
}

spectrum_add!(TSpectrum<T>, TSpectrum<T>);
spectrum_add!(TSpectrum<T>, &TSpectrum<T>);
spectrum_add!(&TSpectrum<T>, TSpectrum<T>);
spectrum_add!(&TSpectrum<T>, &TSpectrum<T>);

macro_rules! spectrum_mul {
    ( $lhs:ty, $rhs:ty ) => {
        impl<T: RealField> std::ops::Mul<$rhs> for $lhs {
            type Output = TSpectrum<T>;

            fn mul(self, rhs: $rhs) -> Self::Output {
                let lhs_len = self.samples.len();
                let rhs_len = rhs.samples.len();
                let mut samples = Vec::with_capacity(lhs_len.max(rhs_len));
                let mut lhs_iter = 0;
                let mut rhs_iter = 0;

                while lhs_iter < lhs_len && rhs_iter < rhs_len {
                    let lhs_sample = &self.samples[lhs_iter];
                    let rhs_sample = &rhs.samples[rhs_iter];
                    if lhs_sample.wavelength < rhs_sample.wavelength {
                        samples.push(Sample::new(lhs_sample.wavelength, T::zero()));
                        lhs_iter += 1;
                    } else if lhs_sample.wavelength == rhs_sample.wavelength {
                        samples.push(Sample::new(
                            lhs_sample.wavelength,
                            lhs_sample.intensity * rhs_sample.intensity,
                        ));
                        lhs_iter += 1;
                        rhs_iter += 1;
                    } else {
                        samples.push(Sample::new(rhs_sample.wavelength, T::zero()));
                        rhs_iter += 1;
                    }
                }

                while lhs_iter < lhs_len {
                    let lhs_sample = &self.samples[lhs_iter];
                    samples.push(Sample::new(lhs_sample.wavelength, T::zero()));
                    lhs_iter += 1;
                }

                while rhs_iter < rhs_len {
                    let rhs_sample = &rhs.samples[rhs_iter];
                    samples.push(Sample::new(rhs_sample.wavelength, T::zero()));
                    rhs_iter += 1;
                }

                Self::Output::new(samples)
            }
        }
    };
}

spectrum_mul!(TSpectrum<T>, TSpectrum<T>);
spectrum_mul!(TSpectrum<T>, &TSpectrum<T>);
spectrum_mul!(&TSpectrum<T>, TSpectrum<T>);
spectrum_mul!(&TSpectrum<T>, &TSpectrum<T>);

impl<T: RealField> std::ops::Mul<T> for TSpectrum<T> {
    type Output = TSpectrum<T>;

    fn mul(mut self, rhs: T) -> Self::Output {
        self.samples.iter_mut().for_each(|sample| {
            sample.intensity *= rhs;
        });
        self
    }
}

impl<T: RealField> std::ops::Mul<T> for &TSpectrum<T> {
    type Output = TSpectrum<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Self::Output::new(
            self.samples
                .iter()
                .map(|sample| Sample::new(*sample.get_wavelength(), *sample.get_intensity() * rhs))
                .collect(),
        )
    }
}

impl<T: std::fmt::Display> std::fmt::Display for TSpectrum<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.get_samples().iter().format(", "))
    }
}

pub fn cie_xyz_to_srgb(xyz: &glm::DVec3) -> glm::DVec3 {
    glm::mat3(
        3.2408302291321256,
        -0.9692293208748544,
        0.05564528732689767,
        -1.5373169035626748,
        1.8759397940918867,
        -0.20403272019862467,
        -0.4985892660203271,
        0.04155444365280374,
        1.0572604592110555,
    ) * xyz
}

lazy_static! {
    /// CIE standard illuminant D65 from
    /// <https://web.archive.org/web/20171122140854/http://www.cie.co.at/publ/abst/datatables15_2004/std65.txt> binned at 5nm.
    pub static ref ILLUMINANT_D65: DSpectrum = {
        DSpectrum::new(
            vec![
                Sample::new(300.0, 0.034100),
                Sample::new(305.0, 1.664300),
                Sample::new(310.0, 3.294500),
                Sample::new(315.0, 11.765200),
                Sample::new(320.0, 20.236000),
                Sample::new(325.0, 28.644700),
                Sample::new(330.0, 37.053500),
                Sample::new(335.0, 38.501100),
                Sample::new(340.0, 39.948800),
                Sample::new(345.0, 42.430200),
                Sample::new(350.0, 44.911700),
                Sample::new(355.0, 45.775000),
                Sample::new(360.0, 46.638300),
                Sample::new(365.0, 49.363700),
                Sample::new(370.0, 52.089100),
                Sample::new(375.0, 51.032300),
                Sample::new(380.0, 49.975500),
                Sample::new(385.0, 52.311800),
                Sample::new(390.0, 54.648200),
                Sample::new(395.0, 68.701500),
                Sample::new(400.0, 82.754900),
                Sample::new(405.0, 87.120400),
                Sample::new(410.0, 91.486000),
                Sample::new(415.0, 92.458900),
                Sample::new(420.0, 93.431800),
                Sample::new(425.0, 90.057000),
                Sample::new(430.0, 86.682300),
                Sample::new(435.0, 95.773600),
                Sample::new(440.0, 104.865000),
                Sample::new(445.0, 110.936000),
                Sample::new(450.0, 117.008000),
                Sample::new(455.0, 117.410000),
                Sample::new(460.0, 117.812000),
                Sample::new(465.0, 116.336000),
                Sample::new(470.0, 114.861000),
                Sample::new(475.0, 115.392000),
                Sample::new(480.0, 115.923000),
                Sample::new(485.0, 112.367000),
                Sample::new(490.0, 108.811000),
                Sample::new(495.0, 109.082000),
                Sample::new(500.0, 109.354000),
                Sample::new(505.0, 108.578000),
                Sample::new(510.0, 107.802000),
                Sample::new(515.0, 106.296000),
                Sample::new(520.0, 104.790000),
                Sample::new(525.0, 106.239000),
                Sample::new(530.0, 107.689000),
                Sample::new(535.0, 106.047000),
                Sample::new(540.0, 104.405000),
                Sample::new(545.0, 104.225000),
                Sample::new(550.0, 104.046000),
                Sample::new(555.0, 102.023000),
                Sample::new(560.0, 100.000000),
                Sample::new(565.0, 98.167100),
                Sample::new(570.0, 96.334200),
                Sample::new(575.0, 96.061100),
                Sample::new(580.0, 95.788000),
                Sample::new(585.0, 92.236800),
                Sample::new(590.0, 88.685600),
                Sample::new(595.0, 89.345900),
                Sample::new(600.0, 90.006200),
                Sample::new(605.0, 89.802600),
                Sample::new(610.0, 89.599100),
                Sample::new(615.0, 88.648900),
                Sample::new(620.0, 87.698700),
                Sample::new(625.0, 85.493600),
                Sample::new(630.0, 83.288600),
                Sample::new(635.0, 83.493900),
                Sample::new(640.0, 83.699200),
                Sample::new(645.0, 81.863000),
                Sample::new(650.0, 80.026800),
                Sample::new(655.0, 80.120700),
                Sample::new(660.0, 80.214600),
                Sample::new(665.0, 81.246200),
                Sample::new(670.0, 82.277800),
                Sample::new(675.0, 80.281000),
                Sample::new(680.0, 78.284200),
                Sample::new(685.0, 74.002700),
                Sample::new(690.0, 69.721300),
                Sample::new(695.0, 70.665200),
                Sample::new(700.0, 71.609100),
                Sample::new(705.0, 72.979000),
                Sample::new(710.0, 74.349000),
                Sample::new(715.0, 67.976500),
                Sample::new(720.0, 61.604000),
                Sample::new(725.0, 65.744800),
                Sample::new(730.0, 69.885600),
                Sample::new(735.0, 72.486300),
                Sample::new(740.0, 75.087000),
                Sample::new(745.0, 69.339800),
                Sample::new(750.0, 63.592700),
                Sample::new(755.0, 55.005400),
                Sample::new(760.0, 46.418200),
                Sample::new(765.0, 56.611800),
                Sample::new(770.0, 66.805400),
                Sample::new(775.0, 65.094100),
                Sample::new(780.0, 63.382800),
            ])
    };

    /// reference: javascript of
    /// <https://geometrian.com/data/research/spectral-primaries/primaries-visualization.html>
    /// variable s_r1
    pub static ref SRGB_R_SPECTRUM: DSpectrum = {
        DSpectrum::new(
            vec![
                Sample::new(380.0, 0.327457414),
                Sample::new(385.0, 0.323750578),
                Sample::new(390.0, 0.313439461),
                Sample::new(395.0, 0.288879383),
                Sample::new(400.0, 0.239205681),
                Sample::new(405.0, 0.189702037),
                Sample::new(410.0, 0.121746068),
                Sample::new(415.0, 0.074578271),
                Sample::new(420.0, 0.044433159),
                Sample::new(425.0, 0.028928632),
                Sample::new(430.0, 0.022316653),
                Sample::new(435.0, 0.016911307),
                Sample::new(440.0, 0.014181107),
                Sample::new(445.0, 0.013053143),
                Sample::new(450.0, 0.011986164),
                Sample::new(455.0, 0.011288715),
                Sample::new(460.0, 0.010906066),
                Sample::new(465.0, 0.010400713),
                Sample::new(470.0, 0.01063736),
                Sample::new(475.0, 0.010907663),
                Sample::new(480.0, 0.011032712),
                Sample::new(485.0, 0.011310657),
                Sample::new(490.0, 0.011154642),
                Sample::new(495.0, 0.01014877),
                Sample::new(500.0, 0.008918582),
                Sample::new(505.0, 0.007685576),
                Sample::new(510.0, 0.006705708),
                Sample::new(515.0, 0.005995806),
                Sample::new(520.0, 0.005537257),
                Sample::new(525.0, 0.005193784),
                Sample::new(530.0, 0.005025362),
                Sample::new(535.0, 0.005136363),
                Sample::new(540.0, 0.0054332),
                Sample::new(545.0, 0.005819986),
                Sample::new(550.0, 0.006400573),
                Sample::new(555.0, 0.007449529),
                Sample::new(560.0, 0.008583636),
                Sample::new(565.0, 0.010395762),
                Sample::new(570.0, 0.013565434),
                Sample::new(575.0, 0.019384516),
                Sample::new(580.0, 0.032084071),
                Sample::new(585.0, 0.074356038),
                Sample::new(590.0, 0.624393724),
                Sample::new(595.0, 0.918310033),
                Sample::new(600.0, 0.94925303),
                Sample::new(605.0, 0.958187833),
                Sample::new(610.0, 0.958187751),
                Sample::new(615.0, 0.958187625),
                Sample::new(620.0, 0.955679061),
                Sample::new(625.0, 0.958006155),
                Sample::new(630.0, 0.954101573),
                Sample::new(635.0, 0.947607606),
                Sample::new(640.0, 0.938681328),
                Sample::new(645.0, 0.924466683),
                Sample::new(650.0, 0.904606025),
                Sample::new(655.0, 0.880412199),
                Sample::new(660.0, 0.847787873),
                Sample::new(665.0, 0.805779127),
                Sample::new(670.0, 0.752531854),
                Sample::new(675.0, 0.686439397),
                Sample::new(680.0, 0.618694571),
                Sample::new(685.0, 0.540264444),
                Sample::new(690.0, 0.472964416),
                Sample::new(695.0, 0.432701597),
                Sample::new(700.0, 0.405358046),
                Sample::new(705.0, 0.385491835),
                Sample::new(710.0, 0.370983585),
                Sample::new(715.0, 0.357608702),
                Sample::new(720.0, 0.3487128),
                Sample::new(725.0, 0.344880119),
                Sample::new(730.0, 0.341917877),
                Sample::new(735.0, 0.339531093),
                Sample::new(740.0, 0.337169504),
                Sample::new(745.0, 0.336172019),
                Sample::new(750.0, 0.335167443),
                Sample::new(755.0, 0.334421625),
                Sample::new(760.0, 0.33400876),
                Sample::new(765.0, 0.333915793),
                Sample::new(770.0, 0.333818455),
                Sample::new(775.0, 0.333672775),
                Sample::new(780.0, 0.333569513),
            ])
    };

    /// reference: javascript of
    /// <https://geometrian.com/data/research/spectral-primaries/primaries-visualization.html>
    /// variable s_g1
    pub static ref SRGB_G_SPECTRUM: DSpectrum = {
        DSpectrum::new(
            vec![
                Sample::new(380.0, 0.331861713),
                Sample::new(385.0, 0.329688188),
                Sample::new(390.0, 0.327860022),
                Sample::new(395.0, 0.31917358),
                Sample::new(400.0, 0.294322584),
                Sample::new(405.0, 0.258697065),
                Sample::new(410.0, 0.188894319),
                Sample::new(415.0, 0.125388382),
                Sample::new(420.0, 0.07868706),
                Sample::new(425.0, 0.053143271),
                Sample::new(430.0, 0.042288146),
                Sample::new(435.0, 0.033318346),
                Sample::new(440.0, 0.029755948),
                Sample::new(445.0, 0.030331251),
                Sample::new(450.0, 0.030988572),
                Sample::new(455.0, 0.031686355),
                Sample::new(460.0, 0.034669962),
                Sample::new(465.0, 0.034551957),
                Sample::new(470.0, 0.040684806),
                Sample::new(475.0, 0.054460037),
                Sample::new(480.0, 0.080905287),
                Sample::new(485.0, 0.146348303),
                Sample::new(490.0, 0.379679643),
                Sample::new(495.0, 0.766744269),
                Sample::new(500.0, 0.876214748),
                Sample::new(505.0, 0.918491656),
                Sample::new(510.0, 0.940655563),
                Sample::new(515.0, 0.953731885),
                Sample::new(520.0, 0.96164328),
                Sample::new(525.0, 0.96720002),
                Sample::new(530.0, 0.970989746),
                Sample::new(535.0, 0.972852304),
                Sample::new(540.0, 0.973116594),
                Sample::new(545.0, 0.973351069),
                Sample::new(550.0, 0.973351116),
                Sample::new(555.0, 0.97226108),
                Sample::new(560.0, 0.973351022),
                Sample::new(565.0, 0.973148495),
                Sample::new(570.0, 0.971061306),
                Sample::new(575.0, 0.966371306),
                Sample::new(580.0, 0.954941968),
                Sample::new(585.0, 0.91357899),
                Sample::new(590.0, 0.364348804),
                Sample::new(595.0, 0.071507243),
                Sample::new(600.0, 0.041230434),
                Sample::new(605.0, 0.032423874),
                Sample::new(610.0, 0.03192463),
                Sample::new(615.0, 0.031276033),
                Sample::new(620.0, 0.03263037),
                Sample::new(625.0, 0.029530872),
                Sample::new(630.0, 0.031561761),
                Sample::new(635.0, 0.035674218),
                Sample::new(640.0, 0.041403005),
                Sample::new(645.0, 0.05060426),
                Sample::new(650.0, 0.0634343),
                Sample::new(655.0, 0.078918245),
                Sample::new(660.0, 0.099542743),
                Sample::new(665.0, 0.12559576),
                Sample::new(670.0, 0.15759091),
                Sample::new(675.0, 0.195398239),
                Sample::new(680.0, 0.231474475),
                Sample::new(685.0, 0.268852136),
                Sample::new(690.0, 0.296029164),
                Sample::new(695.0, 0.309754994),
                Sample::new(700.0, 0.317815883),
                Sample::new(705.0, 0.322990347),
                Sample::new(710.0, 0.326353848),
                Sample::new(715.0, 0.329143902),
                Sample::new(720.0, 0.330808727),
                Sample::new(725.0, 0.33148269),
                Sample::new(730.0, 0.33198455),
                Sample::new(735.0, 0.332341173),
                Sample::new(740.0, 0.332912009),
                Sample::new(745.0, 0.33291928),
                Sample::new(750.0, 0.333027673),
                Sample::new(755.0, 0.333179705),
                Sample::new(760.0, 0.333247031),
                Sample::new(765.0, 0.333259349),
                Sample::new(770.0, 0.33327505),
                Sample::new(775.0, 0.333294328),
                Sample::new(780.0, 0.333309425),
            ])
    };

    /// reference: javascript of
    /// <https://geometrian.com/data/research/spectral-primaries/primaries-visualization.html>
    /// variable s_b1
    pub static ref SRGB_B_SPECTRUM: DSpectrum = {
        DSpectrum::new(
            vec![
                Sample::new(380.0, 0.340680792),
                Sample::new(385.0, 0.346561187),
                Sample::new(390.0, 0.358700493),
                Sample::new(395.0, 0.391947027),
                Sample::new(400.0, 0.466471731),
                Sample::new(405.0, 0.551600896),
                Sample::new(410.0, 0.689359611),
                Sample::new(415.0, 0.800033347),
                Sample::new(420.0, 0.876879781),
                Sample::new(425.0, 0.917928097),
                Sample::new(430.0, 0.935395201),
                Sample::new(435.0, 0.949770347),
                Sample::new(440.0, 0.956062945),
                Sample::new(445.0, 0.956615607),
                Sample::new(450.0, 0.957025265),
                Sample::new(455.0, 0.957024931),
                Sample::new(460.0, 0.954423973),
                Sample::new(465.0, 0.955047329),
                Sample::new(470.0, 0.948677833),
                Sample::new(475.0, 0.9346323),
                Sample::new(480.0, 0.908062),
                Sample::new(485.0, 0.842341039),
                Sample::new(490.0, 0.609165715),
                Sample::new(495.0, 0.223106961),
                Sample::new(500.0, 0.11486667),
                Sample::new(505.0, 0.073822768),
                Sample::new(510.0, 0.052638729),
                Sample::new(515.0, 0.040272309),
                Sample::new(520.0, 0.032819463),
                Sample::new(525.0, 0.027606196),
                Sample::new(530.0, 0.023984891),
                Sample::new(535.0, 0.022011333),
                Sample::new(540.0, 0.021450205),
                Sample::new(545.0, 0.020828945),
                Sample::new(550.0, 0.020248311),
                Sample::new(555.0, 0.020289391),
                Sample::new(560.0, 0.018065342),
                Sample::new(565.0, 0.016455742),
                Sample::new(570.0, 0.01537326),
                Sample::new(575.0, 0.014244178),
                Sample::new(580.0, 0.012973962),
                Sample::new(585.0, 0.012064974),
                Sample::new(590.0, 0.011257478),
                Sample::new(595.0, 0.010182725),
                Sample::new(600.0, 0.009516535),
                Sample::new(605.0, 0.009388293),
                Sample::new(610.0, 0.009887619),
                Sample::new(615.0, 0.010536342),
                Sample::new(620.0, 0.011690569),
                Sample::new(625.0, 0.012462973),
                Sample::new(630.0, 0.014336665),
                Sample::new(635.0, 0.016718175),
                Sample::new(640.0, 0.019915666),
                Sample::new(645.0, 0.024929056),
                Sample::new(650.0, 0.031959674),
                Sample::new(655.0, 0.040669554),
                Sample::new(660.0, 0.052669382),
                Sample::new(665.0, 0.068625111),
                Sample::new(670.0, 0.089877232),
                Sample::new(675.0, 0.118162359),
                Sample::new(680.0, 0.149830947),
                Sample::new(685.0, 0.190883409),
                Sample::new(690.0, 0.231006403),
                Sample::new(695.0, 0.257543385),
                Sample::new(700.0, 0.276826039),
                Sample::new(705.0, 0.291517773),
                Sample::new(710.0, 0.302662506),
                Sample::new(715.0, 0.313247301),
                Sample::new(720.0, 0.320478325),
                Sample::new(725.0, 0.323636995),
                Sample::new(730.0, 0.326097309),
                Sample::new(735.0, 0.328127369),
                Sample::new(740.0, 0.329917976),
                Sample::new(745.0, 0.330907901),
                Sample::new(750.0, 0.331803633),
                Sample::new(755.0, 0.332396627),
                Sample::new(760.0, 0.332740781),
                Sample::new(765.0, 0.332820857),
                Sample::new(770.0, 0.332901731),
                Sample::new(775.0, 0.333025967),
                Sample::new(780.0, 0.333111083),
            ])
    };
}

impl<T: RealField> std::ops::Mul<T> for &SRGB_R_SPECTRUM {
    type Output = TSpectrum<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Self::Output::new(
            self.samples
                .iter()
                .map(|sample| {
                    Sample::new(
                        T::from_f64(*sample.get_wavelength()).unwrap(),
                        T::from_f64(*sample.get_intensity()).unwrap() * rhs,
                    )
                })
                .collect(),
        )
    }
}

impl<T: RealField> std::ops::Mul<T> for &SRGB_G_SPECTRUM {
    type Output = TSpectrum<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Self::Output::new(
            self.samples
                .iter()
                .map(|sample| {
                    Sample::new(
                        T::from_f64(*sample.get_wavelength()).unwrap(),
                        T::from_f64(*sample.get_intensity()).unwrap() * rhs,
                    )
                })
                .collect(),
        )
    }
}

impl<T: RealField> std::ops::Mul<T> for &SRGB_B_SPECTRUM {
    type Output = TSpectrum<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Self::Output::new(
            self.samples
                .iter()
                .map(|sample| {
                    Sample::new(
                        T::from_f64(*sample.get_wavelength()).unwrap(),
                        T::from_f64(*sample.get_intensity()).unwrap() * rhs,
                    )
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spectrum_is_equal<T: RealField>(spectrum1: &TSpectrum<T>, spectrum2: &TSpectrum<T>) -> bool {
        if spectrum1.len() != spectrum2.len() {
            false
        } else {
            spectrum1
                .get_samples()
                .iter()
                .zip(spectrum2.get_samples().iter())
                .try_for_each(|(sample1, sample2)| {
                    if sample1.get_wavelength() != sample2.get_wavelength() {
                        None
                    } else if (*sample1.get_intensity() - *sample2.get_intensity()).abs()
                        < T::from_f64(0.001).unwrap()
                    {
                        Some(())
                    } else {
                        None
                    }
                })
                .is_some()
        }
    }

    #[test]
    fn spectrum_add_01() {
        let give_spectra = || {
            (
                DSpectrum::new(vec![Sample::new(300.0, 1.0)]),
                DSpectrum::new(vec![Sample::new(300.0, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![Sample::new(300.0, 2.0)]);

        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 + spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 + &spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 + spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 + &spectrum2), &expected));
    }

    #[test]
    fn spectrum_add_02() {
        let give_spectra = || {
            (
                DSpectrum::new(vec![Sample::new(300.0, 1.0)]),
                DSpectrum::new(vec![Sample::new(305.0, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![Sample::new(300.0, 1.0), Sample::new(305.0, 1.0)]);

        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 + spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 + &spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 + spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 + &spectrum2), &expected));
    }

    #[test]
    fn spectrum_add_03() {
        let give_spectra = || {
            (
                DSpectrum::new(vec![Sample::new(305.0, 1.0), Sample::new(310.0, 1.0)]),
                DSpectrum::new(vec![Sample::new(300.0, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![
            Sample::new(300.0, 1.0),
            Sample::new(305.0, 1.0),
            Sample::new(310.0, 1.0),
        ]);

        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 + spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 + &spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 + spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 + &spectrum2), &expected));
    }

    #[test]
    fn spectrum_add_04() {
        let give_spectra = || {
            (
                DSpectrum::new(vec![Sample::new(305.0, 1.0), Sample::new(315.0, 1.0)]),
                DSpectrum::new(vec![Sample::new(300.0, 1.0), Sample::new(310.0, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![
            Sample::new(300.0, 1.0),
            Sample::new(305.0, 1.0),
            Sample::new(310.0, 1.0),
            Sample::new(315.0, 1.0),
        ]);

        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 + spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 + &spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 + spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 + &spectrum2), &expected));
    }

    #[test]
    fn spectrum_mul_01() {
        let give_spectra = || {
            (
                DSpectrum::new(vec![Sample::new(300.0, 1.0)]),
                DSpectrum::new(vec![Sample::new(300.0, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![Sample::new(300.0, 1.0)]);

        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 * spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 * &spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 * spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 * &spectrum2), &expected));
    }

    #[test]
    fn spectrum_mul_02() {
        let give_spectra = || {
            (
                DSpectrum::new(vec![Sample::new(300.0, 1.0)]),
                DSpectrum::new(vec![Sample::new(305.0, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![Sample::new(300.0, 0.0), Sample::new(305.0, 0.0)]);

        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 * spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 * &spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 * spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 * &spectrum2), &expected));
    }

    #[test]
    fn spectrum_mul_03() {
        let give_spectra = || {
            (
                DSpectrum::new(vec![Sample::new(305.0, 1.0), Sample::new(310.0, 1.0)]),
                DSpectrum::new(vec![Sample::new(300.0, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![
            Sample::new(300.0, 0.0),
            Sample::new(305.0, 0.0),
            Sample::new(310.0, 0.0),
        ]);

        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 * spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 * &spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 * spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 * &spectrum2), &expected));
    }

    #[test]
    fn spectrum_mul_04() {
        let give_spectra = || {
            (
                DSpectrum::new(vec![Sample::new(305.0, 1.0), Sample::new(315.0, 1.0)]),
                DSpectrum::new(vec![Sample::new(300.0, 1.0), Sample::new(310.0, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![
            Sample::new(300.0, 0.0),
            Sample::new(305.0, 0.0),
            Sample::new(310.0, 0.0),
            Sample::new(315.0, 0.0),
        ]);

        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 * spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 * &spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 * spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 * &spectrum2), &expected));
    }

    #[test]
    fn spectrum_mul_05() {
        let give_spectra = || {
            (
                DSpectrum::new(vec![Sample::new(315.0, 1.0)]),
                DSpectrum::new(vec![Sample::new(300.0, 2.0), Sample::new(315.0, 2.0)]),
            )
        };

        let expected = DSpectrum::new(vec![Sample::new(300.0, 0.0), Sample::new(315.0, 2.0)]);

        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 * spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(spectrum1 * &spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 * spectrum2), &expected));
        let (spectrum1, spectrum2) = give_spectra();
        assert!(spectrum_is_equal(&(&spectrum1 * &spectrum2), &expected));
    }

    #[test]
    fn spectrum_from_srgb_01() {
        let srgb = glm::vec3(0.0, 0.0, 0.0);
        let spectrum = DSpectrum::from_srgb(&srgb);
        assert!(spectrum_is_equal(
            &spectrum,
            &DSpectrum::zeros(380..=780, 5)
        ));
    }

    #[test]
    fn spectrum_from_srgb_02() {
        let srgb = glm::vec3(1.0, 1.0, 1.0);
        let spectrum = DSpectrum::from_srgb(&srgb);
        assert!(spectrum_is_equal(&spectrum, &DSpectrum::ones(380..=780, 5)));
    }
}
