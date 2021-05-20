use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nalgebra_glm as glm;
use rand::prelude::*;

use rt::intersectable::Intersectable;
use rt::math::Scalar;
use rt::ray::Ray;
use rt::sphere::Sphere;
use rt::threadpool::ThreadPool;

struct SceneCollisionParams<'a> {
    ray: &'a Ray,
    scene: &'a Vec<Box<dyn Intersectable + Send + Sync>>,
    t_min: Scalar,
    t_max: Scalar,
}

impl<'a> SceneCollisionParams<'a> {
    fn new(
        ray: &'a Ray,
        scene: &'a Vec<Box<dyn Intersectable + Send + Sync>>,
        t_min: Scalar,
        t_max: Scalar,
    ) -> Self {
        return Self {
            ray,
            scene,
            t_min,
            t_max,
        };
    }
}

fn scene_collision_st(scene_collision_params: &SceneCollisionParams) {
    let ray = scene_collision_params.ray;
    let scene = scene_collision_params.scene;
    let t_min = scene_collision_params.t_min;
    let t_max = scene_collision_params.t_max;
    scene.iter().for_each(|object| {
        object.hit(ray, t_min, t_max);
    });
}

fn scene_collision_mt(scene_collision_params: &SceneCollisionParams) {
    let ray = scene_collision_params.ray;
    let scene = scene_collision_params.scene;
    let t_min = scene_collision_params.t_min;
    let t_max = scene_collision_params.t_max;
    ThreadPool::new_scoped(12, |scope| {
        scene.iter().for_each(|object| {
            scope.execute(|| {
                object.hit(ray, t_min, t_max);
            });
        })
    });
}

fn scene_collision_mt_2(scene_collision_params: &SceneCollisionParams, pool: &ThreadPool) {
    let ray = scene_collision_params.ray;
    let scene = scene_collision_params.scene;
    let t_min = scene_collision_params.t_min;
    let t_max = scene_collision_params.t_max;
    pool.scoped(|scope| {
        scope.execute(|| {
            scene.iter().for_each(|object| {
                object.hit(ray, t_min, t_max);
            })
        });
    });
}

fn scene_collision_mt_3(scene_collision_params: &SceneCollisionParams, pool: &ThreadPool) {
    let ray = scene_collision_params.ray;
    let scene = scene_collision_params.scene;
    let t_min = scene_collision_params.t_min;
    let t_max = scene_collision_params.t_max;
    let mut chunk_size = scene.len() / pool.get_num_threads();
    if chunk_size == 0 {
        chunk_size = 1;
    }
    pool.scoped(|scope| {
        for objects in scene.chunks(chunk_size) {
            scope.execute(|| {
                objects.iter().for_each(|object| {
                    object.hit(ray, t_min, t_max);
                })
            });
        }
    });
}

fn rng_scaled(rng: &mut ThreadRng, scale_factor: f64) -> f64 {
    return (rng.gen::<f64>() - 0.5) * 2.0 * scale_factor;
}

fn random_vec3(rng: &mut ThreadRng, scale_factor: f64) -> glm::DVec3 {
    let x = rng_scaled(rng, scale_factor);
    let y = rng_scaled(rng, scale_factor);
    let z = rng_scaled(rng, scale_factor);
    return glm::vec3(x, y, z);
}

fn bench_scene_collision(c: &mut Criterion) {
    let ray = Ray::new(glm::vec3(0.0, 0.0, 0.0), glm::vec3(0.0, 0.0, -1.0));
    let mut scene: Vec<Box<dyn Intersectable + Send + Sync>> =
        vec![Box::new(Sphere::new(glm::vec3(0.0, 0.0, -2.0), 1.5))];
    let mut rng = rand::thread_rng();
    let num_objects = 12 * 1000 - 1;
    let scale_factor = 5.0;
    for _ in 0..num_objects {
        scene.push(Box::new(Sphere::new(
            random_vec3(&mut rng, scale_factor),
            rng_scaled(&mut rng, scale_factor),
        )));
    }
    let (t_min, t_max) = (0.01, 1000.0);
    let scene_collision_params = SceneCollisionParams::new(&ray, &scene, t_min, t_max);
    let mut group = c.benchmark_group("SceneCollision");
    group.bench_with_input(
        BenchmarkId::new("single_threaded", 1),
        &scene_collision_params,
        |b, scene_collision_params| {
            b.iter(|| scene_collision_st(black_box(scene_collision_params)));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("multi_threaded", 1),
        &scene_collision_params,
        |b, scene_collision_params| {
            b.iter(|| scene_collision_mt(black_box(scene_collision_params)));
        },
    );
    let pool = ThreadPool::new(12);
    group.bench_with_input(
        BenchmarkId::new("multi_threaded", 2),
        &scene_collision_params,
        |b, scene_collision_params| {
            b.iter(|| scene_collision_mt_2(black_box(scene_collision_params), &pool));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("multi_threaded", 3),
        &scene_collision_params,
        |b, scene_collision_params| {
            b.iter(|| scene_collision_mt_3(black_box(scene_collision_params), &pool));
        },
    );
    group.finish();
}

criterion_group!(benches, bench_scene_collision);
criterion_main!(benches);
