use criterion::{criterion_group, criterion_main, Criterion};

use rt::bvh::BVHTree;
use rt::glm;

fn build_scene(bvh_epsilon: f64, bvh_tree_type: u8, bvh_axis: u8) -> BVHTree<usize> {
    let cube_verts = vec![
        glm::vec3(1.0, 1.0, 1.0),
        glm::vec3(1.0, 1.0, -1.0),
        glm::vec3(1.0, -1.0, 1.0),
        glm::vec3(1.0, -1.0, -1.0),
        glm::vec3(-1.0, 1.0, 1.0),
        glm::vec3(-1.0, 1.0, -1.0),
        glm::vec3(-1.0, -1.0, 1.0),
        glm::vec3(-1.0, -1.0, -1.0),
    ];

    let cube = vec![
        [cube_verts[0], cube_verts[3], cube_verts[1]],
        [cube_verts[0], cube_verts[2], cube_verts[3]],
        [cube_verts[2], cube_verts[7], cube_verts[3]],
        [cube_verts[2], cube_verts[6], cube_verts[3]],
        [cube_verts[4], cube_verts[2], cube_verts[0]],
        [cube_verts[4], cube_verts[6], cube_verts[2]],
        [cube_verts[3], cube_verts[5], cube_verts[7]],
        [cube_verts[3], cube_verts[1], cube_verts[5]],
        [cube_verts[5], cube_verts[7], cube_verts[6]],
        [cube_verts[5], cube_verts[1], cube_verts[4]],
        [cube_verts[1], cube_verts[5], cube_verts[4]],
        [cube_verts[1], cube_verts[4], cube_verts[0]],
    ];

    let plane_verts = vec![
        glm::vec3(4.0, -1.0, 4.0),
        glm::vec3(4.0, -1.0, -4.0),
        glm::vec3(-4.0, -1.0, 4.0),
        glm::vec3(-4.0, -1.0, -4.0),
    ];

    let plane = vec![
        [plane_verts[0], plane_verts[2], plane_verts[3]],
        [plane_verts[0], plane_verts[3], plane_verts[1]],
    ];

    let mut bvh = BVHTree::<usize>::new(
        cube.len() + plane.len(),
        bvh_epsilon,
        bvh_tree_type,
        bvh_axis,
    );
    cube.iter()
        .chain(plane.iter())
        .enumerate()
        .for_each(|(index, verts)| {
            bvh.insert(index, verts);
        });
    bvh.balance();
    bvh
}

fn bvh_ray_cast_benchmark(c: &mut Criterion) {
    let ray_origin = glm::vec3(0.0, 0.0, 0.0);
    let camera_focal_length = 50.0;
    let camera_plane_center = glm::vec3(0.0, 0.0, camera_focal_length);
    let camera_horizontal = glm::vec3(1.0, 0.0, 0.0);
    let camera_vertical = glm::vec3(0.0, 1.0, 0.0);
    let width = 1000;
    let height = 1000;
    let ray_dirs: Vec<glm::DVec3> = (0..width)
        .flat_map(|x| {
            (0..height).map(move |y| {
                let u = (x as f64 / width as f64) * 2.0 - 1.0;
                let v = (y as f64 / height as f64) * 2.0 - 1.0;
                camera_plane_center + u * camera_horizontal + v * camera_vertical
            })
        })
        .collect();

    let bvh_2_6 = build_scene(0.01, 2, 6);
    let bvh_4_6 = build_scene(0.01, 4, 6);
    let bvh_8_6 = build_scene(0.01, 8, 6);
    let bvh_16_6 = build_scene(0.01, 16, 6);
    let bvh_24_6 = build_scene(0.01, 24, 6);
    let bvh_31_6 = build_scene(0.01, 31, 6);

    let bvh_2_8 = build_scene(0.01, 2, 8);
    let bvh_4_8 = build_scene(0.01, 4, 8);
    let bvh_8_8 = build_scene(0.01, 8, 8);
    let bvh_16_8 = build_scene(0.01, 16, 8);
    let bvh_24_8 = build_scene(0.01, 24, 8);
    let bvh_31_8 = build_scene(0.01, 31, 8);

    let bvh_2_14 = build_scene(0.01, 2, 14);
    let bvh_4_14 = build_scene(0.01, 4, 14);
    let bvh_8_14 = build_scene(0.01, 8, 14);
    let bvh_16_14 = build_scene(0.01, 16, 14);
    let bvh_24_14 = build_scene(0.01, 24, 14);
    let bvh_31_14 = build_scene(0.01, 31, 14);

    let bvh_2_26 = build_scene(0.01, 2, 26);
    let bvh_4_26 = build_scene(0.01, 4, 26);
    let bvh_8_26 = build_scene(0.01, 8, 26);
    let bvh_16_26 = build_scene(0.01, 16, 26);
    let bvh_24_26 = build_scene(0.01, 24, 26);
    let bvh_31_26 = build_scene(0.01, 31, 26);

    let mut ray_dirs_index = 0;

    let mut group = c.benchmark_group("BVH Ray Cast");

    {
        group.bench_function("bvh_2_6", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_2_6, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_4_6", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_4_6, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_8_6", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_8_6, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_16_6", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_16_6, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_24_6", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_24_6, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_31_6", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_31_6, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });
    }

    {
        group.bench_function("bvh_2_8", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_2_8, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_4_8", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_4_8, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_8_8", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_8_8, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_16_8", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_16_8, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_24_8", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_24_8, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_31_8", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_31_8, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });
    }

    {
        group.bench_function("bvh_2_14", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_2_14, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_4_14", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_4_14, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_8_14", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_8_14, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_16_14", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_16_14, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_24_14", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_24_14, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_31_14", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_31_14, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });
    }

    {
        group.bench_function("bvh_2_26", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_2_26, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_4_26", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_4_26, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_8_26", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_8_26, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_16_26", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_16_26, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_24_26", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_24_26, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });

        group.bench_function("bvh_31_26", |b| {
            ray_dirs_index = 0;
            b.iter(|| {
                bvh_ray_cast(&bvh_31_26, ray_origin, ray_dirs[ray_dirs_index]);
                ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
            });
        });
    }

    group.finish();
}

#[inline(always)]
fn bvh_ray_cast(bvh: &BVHTree<usize>, co: glm::DVec3, dir: glm::DVec3) {
    bvh.ray_cast::<fn((&glm::DVec3, &glm::DVec3), usize) -> Option<rt::bvh::RayHitData<usize, ()>>, ()>(criterion::black_box(co), criterion::black_box(dir), criterion::black_box(None));
}

fn bvh_benchmark_config() -> Criterion {
    Criterion::default()
}

criterion_group!(name = benches; config = bvh_benchmark_config(); targets = bvh_ray_cast_benchmark);
criterion_main!(benches);
