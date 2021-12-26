use std::convert::{TryFrom, TryInto};

use blend::Instance;

use super::{id::ID, FromBlend};

#[derive(Debug)]
pub struct Camera {
    id: ID,

    /// CAM_PERSP, CAM_ORTHO or CAM_PANO.
    camera_type: Type,
    clip_start: f32,
    clip_end: f32,
    lens: f32,
    ortho_scale: f32,
    drawsize: f32,
    sensor_x: f32,
    sensor_y: f32,
    shiftx: f32,
    shifty: f32,

    sensor_fit: SensorFit,
}

impl Camera {
    /// Get a reference to the camera's id.
    pub fn get_id(&self) -> &ID {
        &self.id
    }

    /// Get camera's clip start.
    pub fn get_clip_start(&self) -> f32 {
        self.clip_start
    }

    /// Get camera's clip end.
    pub fn get_clip_end(&self) -> f32 {
        self.clip_end
    }

    /// Get camera's lens.
    pub fn get_lens(&self) -> f32 {
        self.lens
    }

    /// Get camera's ortho scale.
    pub fn get_ortho_scale(&self) -> f32 {
        self.ortho_scale
    }

    /// Get camera's drawsize.
    pub fn get_drawsize(&self) -> f32 {
        self.drawsize
    }

    /// Get camera's sensor x.
    pub fn get_sensor_x(&self) -> f32 {
        self.sensor_x
    }

    /// Get camera's sensor y.
    pub fn get_sensor_y(&self) -> f32 {
        self.sensor_y
    }

    /// Get camera's shiftx.
    pub fn get_shiftx(&self) -> f32 {
        self.shiftx
    }

    /// Get camera's shifty.
    pub fn get_shifty(&self) -> f32 {
        self.shifty
    }

    /// Get camera's camera type.
    pub fn get_camera_type(&self) -> Type {
        self.camera_type
    }

    /// Get camera's sensor fit.
    pub fn get_sensor_fit(&self) -> SensorFit {
        self.sensor_fit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Perspective,
    Orthographic,
    Panoramic,
}

impl TryFrom<i8> for Type {
    type Error = ();

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Perspective),
            1 => Ok(Self::Orthographic),
            2 => Ok(Self::Panoramic),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorFit {
    Auto,
    Horizontal,
    Vertical,
}

impl TryFrom<i8> for SensorFit {
    type Error = ();

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Auto),
            1 => Ok(Self::Horizontal),
            2 => Ok(Self::Vertical),
            _ => Err(()),
        }
    }
}

impl FromBlend for Camera {
    fn from_blend_instance(instance: &Instance) -> Option<Self> {
        if !instance.is_valid("id")
            || !instance.is_valid("type")
            // clip_start renamed to clipsta
            || !instance.is_valid("clipsta")
            // clip_end renamed to clipend
            || !instance.is_valid("clipend")
            || !instance.is_valid("lens")
            || !instance.is_valid("ortho_scale")
            || !instance.is_valid("drawsize")
            || !instance.is_valid("sensor_x")
            || !instance.is_valid("sensor_y")
            || !instance.is_valid("shiftx")
            || !instance.is_valid("shifty")
            || !instance.is_valid("sensor_fit")
        {
            println!("something not available, might not be a camera");
            return None;
        }

        Some(Self {
            id: ID::from_blend_instance(&instance.get("id"))?,
            camera_type: instance.get_i8("type").try_into().unwrap(),
            clip_start: instance.get_f32("clipsta"),
            clip_end: instance.get_f32("clipend"),
            lens: instance.get_f32("lens"),
            ortho_scale: instance.get_f32("ortho_scale"),
            drawsize: instance.get_f32("drawsize"),
            sensor_x: instance.get_f32("sensor_x"),
            sensor_y: instance.get_f32("sensor_y"),
            shiftx: instance.get_f32("shiftx"),
            shifty: instance.get_f32("shifty"),
            sensor_fit: instance.get_i8("sensor_fit").try_into().unwrap(),
        })
    }
}
