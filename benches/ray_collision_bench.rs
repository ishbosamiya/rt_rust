use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nalgebra_glm as glm;

use rt::intersectable::Intersectable;
use rt::math::Scalar;
use rt::ray::Ray;
use rt::sphere::Sphere;
use rt::threadpool::ThreadPool;

struct SceneCollisionParams<'a> {
    ray: &'a Ray,
    scene: &'a Vec<Box<dyn Intersectable>>,
    t_min: Scalar,
    t_max: Scalar,
}

impl<'a> SceneCollisionParams<'a> {
    fn new(
        ray: &'a Ray,
        scene: &'a Vec<Box<dyn Intersectable>>,
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

// fn scene_collision_mt(scene_collision_params: &SceneCollisionParams) {
//     let ray = scene_collision_params.ray;
//     let scene = scene_collision_params.scene;
//     let t_min = scene_collision_params.t_min;
//     let t_max = scene_collision_params.t_max;
//     let pool = ThreadPool::new(11);
//     scene.iter().for_each(|object| {
//         pool.execute(|| {
//             object.hit(ray, t_min, t_max);
//             return;
//         });
//     });
// }

fn bench_scene_collision(c: &mut Criterion) {
    let ray = Ray::new(glm::vec3(0.0, 0.0, 0.0), glm::vec3(0.0, 0.0, -1.0));
    let scene: Vec<Box<dyn Intersectable>> =
        vec![Box::new(Sphere::new(glm::vec3(0.0, 0.0, -2.0), 1.5))];
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
    // group.bench_with_input(
    //     BenchmarkId::new("multi_threaded", 2),
    //     &scene_collision_params,
    //     |b, scene_collision_params| {
    //         b.iter(|| scene_collision_mt(black_box(scene_collision_params)));
    //     },
    // );
    group.finish();
}

criterion_group!(benches, bench_scene_collision);
criterion_main!(benches);
