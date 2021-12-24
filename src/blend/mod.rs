use std::io::Read;

use blend::{Blend, Instance};

use self::object::Object;

pub mod id;
pub mod mesh;
pub mod object;

pub trait FromBlend {
    fn from_blend_instance(instance: &Instance) -> Option<Self>
    where
        Self: std::marker::Sized;
}

pub fn get_all_objects(data: impl Read) -> Vec<Object> {
    let blend = Blend::new(data);
    blend
        .get_by_code(*b"OB")
        .iter()
        .filter_map(|instance| Object::from_blend_instance(instance))
        .collect()
}
