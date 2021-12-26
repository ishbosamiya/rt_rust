//! Blend file parsing module
//!
//! # Renaming
//!
//! The blend file may not store the same name as the name of the
//! struct member, for backwards compatibility and ensuring code
//! readability renaming is supported in the blend file. See
//! `dna_rename_defs.h` in Blender's source code for more details
//! about this as well as the renamed values.

use std::io::Read;

use blend::{Blend, Instance};

use self::object::Object;

pub mod camera;
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
