use lazy_static::lazy_static;
use nalgebra::RealField;

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
                    } else if sample1.get_intensity() == sample2.get_intensity() {
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
}
