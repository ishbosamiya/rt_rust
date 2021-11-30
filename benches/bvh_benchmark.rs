use criterion::{criterion_group, criterion_main, Criterion};

use rt::bvh::BVHTree;
use rt::glm;

fn bvh_ray_cast_benchmark(c: &mut Criterion) {
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
        glm::vec3(1.0, 0.0, 1.0),
        glm::vec3(1.0, 0.0, -1.0),
        glm::vec3(-1.0, 0.0, 1.0),
        glm::vec3(-1.0, 0.0, -1.0),
    ];

    let plane = vec![
        [plane_verts[0], plane_verts[2], plane_verts[3]],
        [plane_verts[0], plane_verts[3], plane_verts[1]],
    ];

    let bvh_epsilon = 0.01;
    let bvh_tree_type = 4;
    let bvh_axis = 6;
    let bvh = {
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
    };

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

    let mut ray_dirs_index = 0;
    c.bench_function("bvh_ray_cast", |b| {
        b.iter(|| bvh.ray_cast::<fn((&glm::DVec3, &glm::DVec3), usize) -> Option<rt::bvh::RayHitData<usize, ()>>, ()>(criterion::black_box(ray_origin), criterion::black_box(ray_dirs[ray_dirs_index]), criterion::black_box(None)));
        ray_dirs_index = (ray_dirs_index + 1) % ray_dirs.len();
    });
}

criterion_group!(benches, bvh_ray_cast_benchmark);
criterion_main!(benches);
