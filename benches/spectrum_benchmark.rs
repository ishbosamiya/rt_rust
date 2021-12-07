use criterion::{criterion_group, criterion_main, Criterion};

use rt::glm;
use rt::path_trace::spectrum::{DSpectrum, TSpectrum, Wavelengths};

fn spectrum_from_srgb_benchmark(c: &mut Criterion) {
    let srgb = glm::vec3(0.334442, 0.36321, 0.212456);
    c.bench_function("spectrum_from_srgb", |b| {
        b.iter(|| {
            DSpectrum::from_srgb(&criterion::black_box(srgb));
        })
    });
}

fn spectrum_from_srgb_for_wavelengths_benchmark(c: &mut Criterion) {
    let srgb = glm::vec3(0.334442, 0.36321, 0.212456);
    let wavelengths = Wavelengths::new(vec![380, 450, 700]);
    c.bench_function("spectrum_from_srgb_for_wavelengths", |b| {
        b.iter(|| {
            DSpectrum::from_srgb_for_wavelengths(
                criterion::black_box(&srgb),
                criterion::black_box(&wavelengths),
            );
        })
    });
}

fn spectrum_to_srgb_benchmark(c: &mut Criterion) {
    let srgb = glm::vec3(0.334442, 0.36321, 0.212456);
    let spectrum = DSpectrum::from_srgb(&srgb);
    c.bench_function("spectrum_to_srgb", |b| {
        b.iter(|| {
            TSpectrum::to_srgb(criterion::black_box(&spectrum));
        })
    });
}

fn spectrum_to_cie_xyz_benchmark(c: &mut Criterion) {
    let srgb = glm::vec3(0.334442, 0.36321, 0.212456);
    let spectrum = DSpectrum::from_srgb(&srgb);
    c.bench_function("spectrum_to_cie_xyz", |b| {
        b.iter(|| {
            TSpectrum::to_cie_xyz(criterion::black_box(&spectrum));
        })
    });
}

fn spectrum_benchmark_config() -> Criterion {
    Criterion::default()
}

criterion_group!(name = benches;
                 config = spectrum_benchmark_config();
                 targets =
                 spectrum_from_srgb_benchmark,
                 spectrum_from_srgb_for_wavelengths_benchmark,
                 spectrum_to_srgb_benchmark,
                 spectrum_to_cie_xyz_benchmark
);
criterion_main!(benches);
