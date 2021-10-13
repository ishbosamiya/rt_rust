use lazy_static::lazy_static;
use paste::paste;

use crate::mesh::Mesh;
use crate::meshio::MeshIO;

macro_rules! load_builtin_mesh {
    ( $name:ident ; $get_str_name:ident ; $get_name:ident ; $static_name:ident ; $location:tt ) => {
        lazy_static! {
            static ref $static_name: Mesh = {
                let file = $get_str_name();
                let lines: Vec<&str> = file.split('\n').collect();
                let reader = MeshIO::from_lines(&lines).unwrap();
                let mut mesh = Mesh::read(&reader).unwrap();
                mesh.build_bvh(0.01);
                mesh.calculate_tangent_info();
                mesh
            };
        }

        pub fn $get_str_name() -> &'static str {
            include_str!($location)
        }
        pub fn $get_name() -> &'static Mesh {
            &$static_name
        }
    };
}

macro_rules! load_builtin_mesh_easy {
    ( $name:ident ; $location:tt ) => {
        paste! {
            load_builtin_mesh!($name; [<get_ $name _obj_str>]; [<get_ $name>]; [<$name:upper>]; $location);
        }
    }
}

load_builtin_mesh_easy!(cube_subd_00; "../../models/cube_subd_00.obj");
load_builtin_mesh_easy!(cube_subd_00_triangulated; "../../models/cube_subd_00_triangulated.obj");

load_builtin_mesh_easy!(ico_sphere_subd_00; "../../models/ico_sphere_subd_00.obj");
load_builtin_mesh_easy!(ico_sphere_subd_01; "../../models/ico_sphere_subd_01.obj");
load_builtin_mesh_easy!(ico_sphere_subd_02; "../../models/ico_sphere_subd_02.obj");

load_builtin_mesh_easy!(monkey_subd_00; "../../models/monkey_subd_00.obj");
load_builtin_mesh_easy!(monkey_subd_00_triangulated; "../../models/monkey_subd_00_triangulated.obj");

load_builtin_mesh_easy!(monkey_subd_01; "../../models/monkey_subd_01.obj");
load_builtin_mesh_easy!(monkey_subd_01_triangulated; "../../models/monkey_subd_01_triangulated.obj");

load_builtin_mesh_easy!(plane_subd_00; "../../models/plane_subd_00.obj");
load_builtin_mesh_easy!(plane_subd_00_triangulated; "../../models/plane_subd_00_triangulated.obj");
