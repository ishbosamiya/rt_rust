use std::sync::{Arc, RwLock};

use criterion::{criterion_group, criterion_main, Criterion};

use rt::inputs::InputArguments;
use rt::path_trace;
use rt::progress::Progress;

fn ray_trace_scene_benchmark(c: &mut Criterion) {
    let arguments =
        InputArguments::read_string("--width 1000 --height 1000 --trace-max-depth 25 --samples 1 --environment ./test_scenes/hdrs/white_background.hdr --textures ./test_scenes/materialball/materialball_scene_backdrop.png --shader-texture backdrop,0 --obj-files ./test_scenes/materialball/materialball_scene.obj --object-shader backdrop,backdrop --object-shader light,emission --object-shader diffuse_object,diffuse --object-shader material_object,test_material --rt-file ./test_scenes/materialball/diffuse.rt".to_string());

    let (ray_trace_params, scene, shader_list, texture_list, environment) =
        arguments.generate_render_info();

    assert_eq!(ray_trace_params.get_width(), 1000);
    assert_eq!(ray_trace_params.get_height(), 1000);
    assert_eq!(ray_trace_params.get_trace_max_depth(), 25);
    assert_eq!(ray_trace_params.get_samples_per_pixel(), 1);

    scene.write().unwrap().rebuild_bvh_if_needed(0.01);

    c.bench_function("ray_trace_scene", |b| {
        b.iter(|| {
            path_trace::ray_trace_scene(
                criterion::black_box(ray_trace_params.clone()),
                criterion::black_box(scene.clone()),
                criterion::black_box(shader_list.clone()),
                criterion::black_box(texture_list.clone()),
                criterion::black_box(environment.clone()),
                criterion::black_box(Arc::new(RwLock::new(Progress::new()))),
                criterion::black_box(Arc::new(RwLock::new(false))),
                criterion::black_box(Arc::new(RwLock::new(false))),
                criterion::black_box(true),
            )
        })
    });
}

fn ray_trace_scene_benchmark_config() -> Criterion {
    Criterion::default().sample_size(20)
}

criterion_group!(name = benches; config = ray_trace_scene_benchmark_config(); targets = ray_trace_scene_benchmark);
criterion_main!(benches);
