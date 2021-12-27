//! Blend file parsing module
//!
//! # Renaming
//!
//! The blend file may not store the same name as the name of the
//! struct member, for backwards compatibility and ensuring code
//! readability renaming is supported in the blend file. See
//! `dna_rename_defs.h` in Blender's source code for more details
//! about this as well as the renamed values.

use std::{fs::File, io::Read, path::Path};

use blend::{Blend, Instance};

use crate::util;

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

pub fn load_blend_data_from_path(path: impl AsRef<Path>) -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open(&path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(if data[0..7] != *b"BLENDER" {
        if util::file_magic_is_gzip(&data) {
            let mut zip_archive = zip::read::ZipArchive::new(std::io::Cursor::new(data)).unwrap();
            let mut unzipped_data = Vec::new();
            zip_archive
                .by_index(0)
                .unwrap()
                .read_to_end(&mut unzipped_data)?;
            unzipped_data
        } else if util::file_magic_is_zstd(&data) {
            zstd::decode_all(std::io::Cursor::new(data))?
        } else {
            panic!("blend file compressed using unknown compression technique");
        }
    } else {
        data
    })
}

pub fn get_all_objects_from_path(path: impl AsRef<Path>) -> Result<Vec<Object>, std::io::Error> {
    Ok(get_all_objects(std::io::Cursor::new(
        load_blend_data_from_path(path)?,
    )))
}

pub fn get_all_objects(data: impl Read) -> Vec<Object> {
    let blend = Blend::new(data);
    blend
        .get_by_code(*b"OB")
        .iter()
        .filter_map(|instance| Object::from_blend_instance(instance))
        .collect()
}
