use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::{
    camera::{Camera, Sensor},
    glm,
    path_trace::{environment::Environment, shader_list::ShaderList},
    scene::Scene,
};

#[derive(Debug, Serialize, Deserialize)]
struct OldPathTraceCamera {
    sensor_height: f64,
    sensor_width: f64,
    aspect_ratio: f64,
    focal_length: f64,
    origin: glm::DVec3,

    horizontal: glm::DVec3,
    vertical: glm::DVec3,
    camera_plane_center: glm::DVec3,
}

impl From<OldPathTraceCamera> for Camera {
    fn from(cam: OldPathTraceCamera) -> Self {
        let mut camera = Camera::new(
            cam.origin,
            glm::vec3(0.0, 1.0, 0.0),
            -90.0,
            0.0,
            45.0,
            Some(Sensor::new(cam.sensor_width, cam.sensor_height)),
        );
        camera.set_focal_length(cam.focal_length);
        camera
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum CameraIO {
    V0(OldPathTraceCamera),
    V1(Camera),
}

impl From<CameraIO> for Camera {
    fn from(io: CameraIO) -> Self {
        match io {
            CameraIO::V0(cam) => cam.into(),
            CameraIO::V1(cam) => cam,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(from = "FileShadow")]
pub struct File {
    scene: Arc<RwLock<Scene>>,
    shader_list: Arc<RwLock<ShaderList>>,
    path_trace_camera: Arc<RwLock<Camera>>,

    environment: Arc<RwLock<Environment>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileShadow {
    scene: Arc<RwLock<Scene>>,
    shader_list: Arc<RwLock<ShaderList>>,
    path_trace_camera: Arc<RwLock<CameraIO>>,

    #[serde(default = "default_environment")]
    environment: Arc<RwLock<Environment>>,
}

impl From<FileShadow> for File {
    fn from(file: FileShadow) -> Self {
        Self {
            scene: file.scene,
            shader_list: file.shader_list,
            path_trace_camera: Arc::new(RwLock::new(
                Arc::try_unwrap(file.path_trace_camera)
                    .unwrap()
                    .into_inner()
                    .unwrap()
                    .into(),
            )),
            environment: file.environment,
        }
    }
}

fn default_environment() -> Arc<RwLock<Environment>> {
    Arc::new(RwLock::new(Environment::default()))
}

impl File {
    pub fn new(
        scene: Arc<RwLock<Scene>>,
        shader_list: Arc<RwLock<ShaderList>>,
        path_trace_camera: Arc<RwLock<Camera>>,
        environment: Arc<RwLock<Environment>>,
    ) -> Self {
        Self {
            scene,
            shader_list,
            path_trace_camera,
            environment,
        }
    }
}

pub fn load_rt_file<P>(
    path: P,
    scene: Arc<RwLock<Scene>>,
    shader_list: Arc<RwLock<ShaderList>>,
    path_trace_camera: Arc<RwLock<Camera>>,
    environment: Arc<RwLock<Environment>>,
) where
    P: AsRef<std::path::Path>,
{
    let json = String::from_utf8(std::fs::read(path).unwrap()).unwrap();
    let file: File = serde_json::from_str(&json).unwrap();
    *scene.write().unwrap() = Arc::try_unwrap(file.scene).unwrap().into_inner().unwrap();
    *shader_list.write().unwrap() = Arc::try_unwrap(file.shader_list)
        .unwrap()
        .into_inner()
        .unwrap();
    *path_trace_camera.write().unwrap() = Arc::try_unwrap(file.path_trace_camera)
        .unwrap()
        .into_inner()
        .unwrap();
    *environment.write().unwrap() = Arc::try_unwrap(file.environment)
        .unwrap()
        .into_inner()
        .unwrap();
}

pub fn save_rt_file<P>(
    path: P,
    scene: Arc<RwLock<Scene>>,
    shader_list: Arc<RwLock<ShaderList>>,
    path_trace_camera: Arc<RwLock<Camera>>,
    environment: Arc<RwLock<Environment>>,
) where
    P: AsRef<std::path::Path>,
{
    let file = File::new(scene, shader_list, path_trace_camera, environment);
    let file_serialized = serde_json::to_string(&file).unwrap();
    std::fs::write(path, file_serialized).unwrap();
}
