use std::collections::HashMap;

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
///
/// Wavelength should always be a positive integer. This makes the
/// rest of the code simpler plus CIE does not have sub nanometer
/// charts anyway.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample<T> {
    wavelength: usize,
    intensity: T,
}

impl<T> Sample<T> {
    pub fn new(wavelength: usize, intensity: T) -> Self {
        Self {
            wavelength,
            intensity,
        }
    }

    /// Get a reference to the sample's wavelength.
    pub fn get_wavelength(&self) -> &usize {
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
    pub fn zeros(wavelength_range: std::ops::RangeInclusive<usize>, step: usize) -> Self {
        Self::new(
            wavelength_range
                .step_by(step)
                .map(|wavelength| Sample::new(wavelength, T::zero()))
                .collect(),
        )
    }

    /// Generate a spectrum with wavelengths of the given range and
    /// step size and their intensities as 1.0
    pub fn ones(wavelength_range: std::ops::RangeInclusive<usize>, step: usize) -> Self {
        Self::new(
            wavelength_range
                .step_by(step)
                .map(|wavelength| Sample::new(wavelength, T::one()))
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
    pub static ref ILLUMINANT_D65: HashMap<usize, f64> = {
        let mut map = HashMap::new();
        map.insert(300, 0.034100);
        map.insert(305, 1.664300);
        map.insert(310, 3.294500);
        map.insert(315, 11.765200);
        map.insert(320, 20.236000);
        map.insert(325, 28.644700);
        map.insert(330, 37.053500);
        map.insert(335, 38.501100);
        map.insert(340, 39.948800);
        map.insert(345, 42.430200);
        map.insert(350, 44.911700);
        map.insert(355, 45.775000);
        map.insert(360, 46.638300);
        map.insert(365, 49.363700);
        map.insert(370, 52.089100);
        map.insert(375, 51.032300);
        map.insert(380, 49.975500);
        map.insert(385, 52.311800);
        map.insert(390, 54.648200);
        map.insert(395, 68.701500);
        map.insert(400, 82.754900);
        map.insert(405, 87.120400);
        map.insert(410, 91.486000);
        map.insert(415, 92.458900);
        map.insert(420, 93.431800);
        map.insert(425, 90.057000);
        map.insert(430, 86.682300);
        map.insert(435, 95.773600);
        map.insert(440, 104.865000);
        map.insert(445, 110.936000);
        map.insert(450, 117.008000);
        map.insert(455, 117.410000);
        map.insert(460, 117.812000);
        map.insert(465, 116.336000);
        map.insert(470, 114.861000);
        map.insert(475, 115.392000);
        map.insert(480, 115.923000);
        map.insert(485, 112.367000);
        map.insert(490, 108.811000);
        map.insert(495, 109.082000);
        map.insert(500, 109.354000);
        map.insert(505, 108.578000);
        map.insert(510, 107.802000);
        map.insert(515, 106.296000);
        map.insert(520, 104.790000);
        map.insert(525, 106.239000);
        map.insert(530, 107.689000);
        map.insert(535, 106.047000);
        map.insert(540, 104.405000);
        map.insert(545, 104.225000);
        map.insert(550, 104.046000);
        map.insert(555, 102.023000);
        map.insert(560, 100.000000);
        map.insert(565, 98.167100);
        map.insert(570, 96.334200);
        map.insert(575, 96.061100);
        map.insert(580, 95.788000);
        map.insert(585, 92.236800);
        map.insert(590, 88.685600);
        map.insert(595, 89.345900);
        map.insert(600, 90.006200);
        map.insert(605, 89.802600);
        map.insert(610, 89.599100);
        map.insert(615, 88.648900);
        map.insert(620, 87.698700);
        map.insert(625, 85.493600);
        map.insert(630, 83.288600);
        map.insert(635, 83.493900);
        map.insert(640, 83.699200);
        map.insert(645, 81.863000);
        map.insert(650, 80.026800);
        map.insert(655, 80.120700);
        map.insert(660, 80.214600);
        map.insert(665, 81.246200);
        map.insert(670, 82.277800);
        map.insert(675, 80.281000);
        map.insert(680, 78.284200);
        map.insert(685, 74.002700);
        map.insert(690, 69.721300);
        map.insert(695, 70.665200);
        map.insert(700, 71.609100);
        map.insert(705, 72.979000);
        map.insert(710, 74.349000);
        map.insert(715, 67.976500);
        map.insert(720, 61.604000);
        map.insert(725, 65.744800);
        map.insert(730, 69.885600);
        map.insert(735, 72.486300);
        map.insert(740, 75.087000);
        map.insert(745, 69.339800);
        map.insert(750, 63.592700);
        map.insert(755, 55.005400);
        map.insert(760, 46.418200);
        map.insert(765, 56.611800);
        map.insert(770, 66.805400);
        map.insert(775, 65.094100);
        map.insert(780, 63.382800);
        map
    };

    /// CIE standard illuminant D65 as [`DSpectrum`] from
    /// <https://web.archive.org/web/20171122140854/http://www.cie.co.at/publ/abst/datatables15_2004/std65.txt> binned at 5nm.
    pub static ref ILLUMINANT_D65_SPECTRUM: DSpectrum = {
        DSpectrum::new(
            vec![
                // Most of the calculations require that the
                // wavelengths be in the range [380, 780], so
                // considering the wavelengths outside this range is
                // not useful.
                //
                // Sample::new(300, 0.034100),
                // Sample::new(305, 1.664300),
                // Sample::new(310, 3.294500),
                // Sample::new(315, 11.765200),
                // Sample::new(320, 20.236000),
                // Sample::new(325, 28.644700),
                // Sample::new(330, 37.053500),
                // Sample::new(335, 38.501100),
                // Sample::new(340, 39.948800),
                // Sample::new(345, 42.430200),
                // Sample::new(350, 44.911700),
                // Sample::new(355, 45.775000),
                // Sample::new(360, 46.638300),
                // Sample::new(365, 49.363700),
                // Sample::new(370, 52.089100),
                // Sample::new(375, 51.032300),
                Sample::new(380, 49.975500),
                Sample::new(385, 52.311800),
                Sample::new(390, 54.648200),
                Sample::new(395, 68.701500),
                Sample::new(400, 82.754900),
                Sample::new(405, 87.120400),
                Sample::new(410, 91.486000),
                Sample::new(415, 92.458900),
                Sample::new(420, 93.431800),
                Sample::new(425, 90.057000),
                Sample::new(430, 86.682300),
                Sample::new(435, 95.773600),
                Sample::new(440, 104.865000),
                Sample::new(445, 110.936000),
                Sample::new(450, 117.008000),
                Sample::new(455, 117.410000),
                Sample::new(460, 117.812000),
                Sample::new(465, 116.336000),
                Sample::new(470, 114.861000),
                Sample::new(475, 115.392000),
                Sample::new(480, 115.923000),
                Sample::new(485, 112.367000),
                Sample::new(490, 108.811000),
                Sample::new(495, 109.082000),
                Sample::new(500, 109.354000),
                Sample::new(505, 108.578000),
                Sample::new(510, 107.802000),
                Sample::new(515, 106.296000),
                Sample::new(520, 104.790000),
                Sample::new(525, 106.239000),
                Sample::new(530, 107.689000),
                Sample::new(535, 106.047000),
                Sample::new(540, 104.405000),
                Sample::new(545, 104.225000),
                Sample::new(550, 104.046000),
                Sample::new(555, 102.023000),
                Sample::new(560, 100.000000),
                Sample::new(565, 98.167100),
                Sample::new(570, 96.334200),
                Sample::new(575, 96.061100),
                Sample::new(580, 95.788000),
                Sample::new(585, 92.236800),
                Sample::new(590, 88.685600),
                Sample::new(595, 89.345900),
                Sample::new(600, 90.006200),
                Sample::new(605, 89.802600),
                Sample::new(610, 89.599100),
                Sample::new(615, 88.648900),
                Sample::new(620, 87.698700),
                Sample::new(625, 85.493600),
                Sample::new(630, 83.288600),
                Sample::new(635, 83.493900),
                Sample::new(640, 83.699200),
                Sample::new(645, 81.863000),
                Sample::new(650, 80.026800),
                Sample::new(655, 80.120700),
                Sample::new(660, 80.214600),
                Sample::new(665, 81.246200),
                Sample::new(670, 82.277800),
                Sample::new(675, 80.281000),
                Sample::new(680, 78.284200),
                Sample::new(685, 74.002700),
                Sample::new(690, 69.721300),
                Sample::new(695, 70.665200),
                Sample::new(700, 71.609100),
                Sample::new(705, 72.979000),
                Sample::new(710, 74.349000),
                Sample::new(715, 67.976500),
                Sample::new(720, 61.604000),
                Sample::new(725, 65.744800),
                Sample::new(730, 69.885600),
                Sample::new(735, 72.486300),
                Sample::new(740, 75.087000),
                Sample::new(745, 69.339800),
                Sample::new(750, 63.592700),
                Sample::new(755, 55.005400),
                Sample::new(760, 46.418200),
                Sample::new(765, 56.611800),
                Sample::new(770, 66.805400),
                Sample::new(775, 65.094100),
                Sample::new(780, 63.382800),
            ])
    };

    /// reference: javascript of
    /// <https://geometrian.com/data/research/spectral-primaries/primaries-visualization.html>
    /// variable s_r1
    pub static ref SRGB_R_SPECTRUM: HashMap<usize, f64> = {
        let mut map = HashMap::new();
        map.insert(380, 0.327457414);
        map.insert(385, 0.323750578);
        map.insert(390, 0.313439461);
        map.insert(395, 0.288879383);
        map.insert(400, 0.239205681);
        map.insert(405, 0.189702037);
        map.insert(410, 0.121746068);
        map.insert(415, 0.074578271);
        map.insert(420, 0.044433159);
        map.insert(425, 0.028928632);
        map.insert(430, 0.022316653);
        map.insert(435, 0.016911307);
        map.insert(440, 0.014181107);
        map.insert(445, 0.013053143);
        map.insert(450, 0.011986164);
        map.insert(455, 0.011288715);
        map.insert(460, 0.010906066);
        map.insert(465, 0.010400713);
        map.insert(470, 0.01063736);
        map.insert(475, 0.010907663);
        map.insert(480, 0.011032712);
        map.insert(485, 0.011310657);
        map.insert(490, 0.011154642);
        map.insert(495, 0.01014877);
        map.insert(500, 0.008918582);
        map.insert(505, 0.007685576);
        map.insert(510, 0.006705708);
        map.insert(515, 0.005995806);
        map.insert(520, 0.005537257);
        map.insert(525, 0.005193784);
        map.insert(530, 0.005025362);
        map.insert(535, 0.005136363);
        map.insert(540, 0.0054332);
        map.insert(545, 0.005819986);
        map.insert(550, 0.006400573);
        map.insert(555, 0.007449529);
        map.insert(560, 0.008583636);
        map.insert(565, 0.010395762);
        map.insert(570, 0.013565434);
        map.insert(575, 0.019384516);
        map.insert(580, 0.032084071);
        map.insert(585, 0.074356038);
        map.insert(590, 0.624393724);
        map.insert(595, 0.918310033);
        map.insert(600, 0.94925303);
        map.insert(605, 0.958187833);
        map.insert(610, 0.958187751);
        map.insert(615, 0.958187625);
        map.insert(620, 0.955679061);
        map.insert(625, 0.958006155);
        map.insert(630, 0.954101573);
        map.insert(635, 0.947607606);
        map.insert(640, 0.938681328);
        map.insert(645, 0.924466683);
        map.insert(650, 0.904606025);
        map.insert(655, 0.880412199);
        map.insert(660, 0.847787873);
        map.insert(665, 0.805779127);
        map.insert(670, 0.752531854);
        map.insert(675, 0.686439397);
        map.insert(680, 0.618694571);
        map.insert(685, 0.540264444);
        map.insert(690, 0.472964416);
        map.insert(695, 0.432701597);
        map.insert(700, 0.405358046);
        map.insert(705, 0.385491835);
        map.insert(710, 0.370983585);
        map.insert(715, 0.357608702);
        map.insert(720, 0.3487128);
        map.insert(725, 0.344880119);
        map.insert(730, 0.341917877);
        map.insert(735, 0.339531093);
        map.insert(740, 0.337169504);
        map.insert(745, 0.336172019);
        map.insert(750, 0.335167443);
        map.insert(755, 0.334421625);
        map.insert(760, 0.33400876);
        map.insert(765, 0.333915793);
        map.insert(770, 0.333818455);
        map.insert(775, 0.333672775);
        map.insert(780, 0.333569513);
        map
    };

    /// reference: javascript of
    /// <https://geometrian.com/data/research/spectral-primaries/primaries-visualization.html>
    /// variable s_g1
    pub static ref SRGB_G_SPECTRUM: HashMap<usize, f64> = {
        let mut map = HashMap::new();
        map.insert(380, 0.331861713);
        map.insert(385, 0.329688188);
        map.insert(390, 0.327860022);
        map.insert(395, 0.31917358);
        map.insert(400, 0.294322584);
        map.insert(405, 0.258697065);
        map.insert(410, 0.188894319);
        map.insert(415, 0.125388382);
        map.insert(420, 0.07868706);
        map.insert(425, 0.053143271);
        map.insert(430, 0.042288146);
        map.insert(435, 0.033318346);
        map.insert(440, 0.029755948);
        map.insert(445, 0.030331251);
        map.insert(450, 0.030988572);
        map.insert(455, 0.031686355);
        map.insert(460, 0.034669962);
        map.insert(465, 0.034551957);
        map.insert(470, 0.040684806);
        map.insert(475, 0.054460037);
        map.insert(480, 0.080905287);
        map.insert(485, 0.146348303);
        map.insert(490, 0.379679643);
        map.insert(495, 0.766744269);
        map.insert(500, 0.876214748);
        map.insert(505, 0.918491656);
        map.insert(510, 0.940655563);
        map.insert(515, 0.953731885);
        map.insert(520, 0.96164328);
        map.insert(525, 0.96720002);
        map.insert(530, 0.970989746);
        map.insert(535, 0.972852304);
        map.insert(540, 0.973116594);
        map.insert(545, 0.973351069);
        map.insert(550, 0.973351116);
        map.insert(555, 0.97226108);
        map.insert(560, 0.973351022);
        map.insert(565, 0.973148495);
        map.insert(570, 0.971061306);
        map.insert(575, 0.966371306);
        map.insert(580, 0.954941968);
        map.insert(585, 0.91357899);
        map.insert(590, 0.364348804);
        map.insert(595, 0.071507243);
        map.insert(600, 0.041230434);
        map.insert(605, 0.032423874);
        map.insert(610, 0.03192463);
        map.insert(615, 0.031276033);
        map.insert(620, 0.03263037);
        map.insert(625, 0.029530872);
        map.insert(630, 0.031561761);
        map.insert(635, 0.035674218);
        map.insert(640, 0.041403005);
        map.insert(645, 0.05060426);
        map.insert(650, 0.0634343);
        map.insert(655, 0.078918245);
        map.insert(660, 0.099542743);
        map.insert(665, 0.12559576);
        map.insert(670, 0.15759091);
        map.insert(675, 0.195398239);
        map.insert(680, 0.231474475);
        map.insert(685, 0.268852136);
        map.insert(690, 0.296029164);
        map.insert(695, 0.309754994);
        map.insert(700, 0.317815883);
        map.insert(705, 0.322990347);
        map.insert(710, 0.326353848);
        map.insert(715, 0.329143902);
        map.insert(720, 0.330808727);
        map.insert(725, 0.33148269);
        map.insert(730, 0.33198455);
        map.insert(735, 0.332341173);
        map.insert(740, 0.332912009);
        map.insert(745, 0.33291928);
        map.insert(750, 0.333027673);
        map.insert(755, 0.333179705);
        map.insert(760, 0.333247031);
        map.insert(765, 0.333259349);
        map.insert(770, 0.33327505);
        map.insert(775, 0.333294328);
        map.insert(780, 0.333309425);
        map
    };

    /// reference: javascript of
    /// <https://geometrian.com/data/research/spectral-primaries/primaries-visualization.html>
    /// variable s_b1
    pub static ref SRGB_B_SPECTRUM: HashMap<usize, f64> = {
        let mut map = HashMap::new();
        map.insert(380, 0.340680792);
        map.insert(385, 0.346561187);
        map.insert(390, 0.358700493);
        map.insert(395, 0.391947027);
        map.insert(400, 0.466471731);
        map.insert(405, 0.551600896);
        map.insert(410, 0.689359611);
        map.insert(415, 0.800033347);
        map.insert(420, 0.876879781);
        map.insert(425, 0.917928097);
        map.insert(430, 0.935395201);
        map.insert(435, 0.949770347);
        map.insert(440, 0.956062945);
        map.insert(445, 0.956615607);
        map.insert(450, 0.957025265);
        map.insert(455, 0.957024931);
        map.insert(460, 0.954423973);
        map.insert(465, 0.955047329);
        map.insert(470, 0.948677833);
        map.insert(475, 0.9346323);
        map.insert(480, 0.908062);
        map.insert(485, 0.842341039);
        map.insert(490, 0.609165715);
        map.insert(495, 0.223106961);
        map.insert(500, 0.11486667);
        map.insert(505, 0.073822768);
        map.insert(510, 0.052638729);
        map.insert(515, 0.040272309);
        map.insert(520, 0.032819463);
        map.insert(525, 0.027606196);
        map.insert(530, 0.023984891);
        map.insert(535, 0.022011333);
        map.insert(540, 0.021450205);
        map.insert(545, 0.020828945);
        map.insert(550, 0.020248311);
        map.insert(555, 0.020289391);
        map.insert(560, 0.018065342);
        map.insert(565, 0.016455742);
        map.insert(570, 0.01537326);
        map.insert(575, 0.014244178);
        map.insert(580, 0.012973962);
        map.insert(585, 0.012064974);
        map.insert(590, 0.011257478);
        map.insert(595, 0.010182725);
        map.insert(600, 0.009516535);
        map.insert(605, 0.009388293);
        map.insert(610, 0.009887619);
        map.insert(615, 0.010536342);
        map.insert(620, 0.011690569);
        map.insert(625, 0.012462973);
        map.insert(630, 0.014336665);
        map.insert(635, 0.016718175);
        map.insert(640, 0.019915666);
        map.insert(645, 0.024929056);
        map.insert(650, 0.031959674);
        map.insert(655, 0.040669554);
        map.insert(660, 0.052669382);
        map.insert(665, 0.068625111);
        map.insert(670, 0.089877232);
        map.insert(675, 0.118162359);
        map.insert(680, 0.149830947);
        map.insert(685, 0.190883409);
        map.insert(690, 0.231006403);
        map.insert(695, 0.257543385);
        map.insert(700, 0.276826039);
        map.insert(705, 0.291517773);
        map.insert(710, 0.302662506);
        map.insert(715, 0.313247301);
        map.insert(720, 0.320478325);
        map.insert(725, 0.323636995);
        map.insert(730, 0.326097309);
        map.insert(735, 0.328127369);
        map.insert(740, 0.329917976);
        map.insert(745, 0.330907901);
        map.insert(750, 0.331803633);
        map.insert(755, 0.332396627);
        map.insert(760, 0.332740781);
        map.insert(765, 0.332820857);
        map.insert(770, 0.332901731);
        map.insert(775, 0.333025967);
        map.insert(780, 0.333111083);
        map
    };
}

macro_rules! mul_srgb_x_spectrum {
    ( $x_spectrum:ty ) => {
        impl<T: RealField> std::ops::Mul<T> for $x_spectrum {
            type Output = TSpectrum<T>;

            fn mul(self, rhs: T) -> Self::Output {
                Self::Output::new(
                    (380..=780)
                        .step_by(5)
                        .map(|wavelength| {
                            Sample::new(
                                wavelength,
                                T::from_f64(*self.get(&wavelength).unwrap()).unwrap() * rhs,
                            )
                        })
                        .collect(),
                )
            }
        }
    };
}

mul_srgb_x_spectrum!(&SRGB_R_SPECTRUM);
mul_srgb_x_spectrum!(&SRGB_G_SPECTRUM);
mul_srgb_x_spectrum!(&SRGB_B_SPECTRUM);

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
                DSpectrum::new(vec![Sample::new(300, 1.0)]),
                DSpectrum::new(vec![Sample::new(300, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![Sample::new(300, 2.0)]);

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
                DSpectrum::new(vec![Sample::new(300, 1.0)]),
                DSpectrum::new(vec![Sample::new(305, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![Sample::new(300, 1.0), Sample::new(305, 1.0)]);

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
                DSpectrum::new(vec![Sample::new(305, 1.0), Sample::new(310, 1.0)]),
                DSpectrum::new(vec![Sample::new(300, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![
            Sample::new(300, 1.0),
            Sample::new(305, 1.0),
            Sample::new(310, 1.0),
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
                DSpectrum::new(vec![Sample::new(305, 1.0), Sample::new(315, 1.0)]),
                DSpectrum::new(vec![Sample::new(300, 1.0), Sample::new(310, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![
            Sample::new(300, 1.0),
            Sample::new(305, 1.0),
            Sample::new(310, 1.0),
            Sample::new(315, 1.0),
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
                DSpectrum::new(vec![Sample::new(300, 1.0)]),
                DSpectrum::new(vec![Sample::new(300, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![Sample::new(300, 1.0)]);

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
                DSpectrum::new(vec![Sample::new(300, 1.0)]),
                DSpectrum::new(vec![Sample::new(305, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![Sample::new(300, 0.0), Sample::new(305, 0.0)]);

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
                DSpectrum::new(vec![Sample::new(305, 1.0), Sample::new(310, 1.0)]),
                DSpectrum::new(vec![Sample::new(300, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![
            Sample::new(300, 0.0),
            Sample::new(305, 0.0),
            Sample::new(310, 0.0),
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
                DSpectrum::new(vec![Sample::new(305, 1.0), Sample::new(315, 1.0)]),
                DSpectrum::new(vec![Sample::new(300, 1.0), Sample::new(310, 1.0)]),
            )
        };

        let expected = DSpectrum::new(vec![
            Sample::new(300, 0.0),
            Sample::new(305, 0.0),
            Sample::new(310, 0.0),
            Sample::new(315, 0.0),
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
                DSpectrum::new(vec![Sample::new(315, 1.0)]),
                DSpectrum::new(vec![Sample::new(300, 2.0), Sample::new(315, 2.0)]),
            )
        };

        let expected = DSpectrum::new(vec![Sample::new(300, 0.0), Sample::new(315, 2.0)]);

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
