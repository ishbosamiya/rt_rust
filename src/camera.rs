use quick_renderer::camera::{Camera, Sensor};

use crate::{blend, glm, path_trace::ray::Ray, util};

pub trait CameraExtension {
    /// Get ray given the UVs on the camera sensor.
    ///
    /// See [`Camera::get_raycast_direction_uv()`] for more details,
    /// this function only makes it easy to get the [`Ray`] directly.
    fn get_ray(&self, uv: &glm::DVec2) -> Option<Ray>;

    /// Create new camera from camera data available in the Camera
    /// Object of a Blend file
    ///
    /// The camera roll is not taken into account, is not supported
    /// yet.
    ///
    /// If the given object is not a camera, [`None`] is returned.
    fn from_blend(camera: &blend::object::Object) -> Option<Self>
    where
        Self: Sized;
}

impl CameraExtension for Camera {
    fn get_ray(&self, uv: &glm::DVec2) -> Option<Ray> {
        Some(Ray::new(
            self.get_position(),
            self.get_raycast_direction_uv(uv)?,
        ))
    }

    fn from_blend(camera: &blend::object::Object) -> Option<Self> {
        let camera_data = match camera.get_data()? {
            blend::id::IDObject::Camera(data) => data,
            _ => return None,
        };

        let camera_sensor_size = match camera_data.get_sensor_fit() {
            blend::camera::SensorFit::Auto => {
                if camera_data.get_sensor_x() > camera_data.get_sensor_y() {
                    camera_data.get_sensor_x()
                } else {
                    camera_data.get_sensor_y()
                }
            }
            blend::camera::SensorFit::Horizontal => camera_data.get_sensor_x(),
            blend::camera::SensorFit::Vertical => camera_data.get_sensor_y(),
        }
        .into();

        let position = util::vec3_apply_model_matrix(
            &glm::convert(glm::make_vec3(camera.get_loc())),
            &util::axis_conversion_matrix_from_blender(),
        );

        let rotation = util::vec3_apply_model_matrix(
            &glm::convert(glm::make_vec3(camera.get_rot())),
            &util::axis_conversion_matrix_from_blender(),
        );

        let roll_pitch_yaw = {
            let roll_pitch_yaw = util::euler_rotation_change_mode(
                &rotation,
                camera.get_rotmode(),
                util::RotationModes::RollPitchYaw,
            );
            glm::vec3(
                roll_pitch_yaw[0].to_degrees(),
                roll_pitch_yaw[1].to_degrees(),
                roll_pitch_yaw[2].to_degrees(),
            )
        };

        let mut res = Self::new(
            position,
            glm::vec3(0.0, 1.0, 0.0),
            roll_pitch_yaw[2],
            roll_pitch_yaw[1],
            util::focal_length_to_fov(camera_data.get_lens().into(), camera_sensor_size)
                .to_degrees(),
            Some(Sensor::new(
                camera_data.get_sensor_x().into(),
                camera_data.get_sensor_y().into(),
            )),
        );
        res.set_near_plane(camera_data.get_clip_start().into());
        res.set_far_plane(camera_data.get_clip_end().into());

        Some(res)
    }
}
