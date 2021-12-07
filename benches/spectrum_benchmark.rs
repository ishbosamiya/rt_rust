use criterion::{criterion_group, criterion_main, Criterion};

use rt::glm;
use rt::path_trace::spectrum::{DSpectrum, TSpectrum};

fn spectrum_from_srgb_benchmark(c: &mut Criterion) {
    let srgb = glm::vec3(0.334442, 0.36321, 0.212456);
    c.bench_function("spectrum_from_srgb", |b| {
        b.iter(|| {
            DSpectrum::from_srgb(&criterion::black_box(srgb));
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

fn spectrum_benchmark_config() -> Criterion {
    Criterion::default()
}

criterion_group!(name = benches;
                 config = spectrum_benchmark_config();
                 targets = spectrum_from_srgb_benchmark, spectrum_to_srgb_benchmark);
criterion_main!(benches);
