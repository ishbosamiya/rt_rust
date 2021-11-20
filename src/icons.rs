use lazy_static::lazy_static;
use paste::paste;

use std::{
    io::Cursor,
    sync::{Arc, RwLock},
};

use crate::rasterize::texture::TextureRGBAFloat;

macro_rules! define_icon {
    ( $name:ident , $location:tt ) => {
        paste! {
            lazy_static! {
                static ref  [<$name:upper>] : Arc<RwLock<TextureRGBAFloat>> = {
                    let icon = TextureRGBAFloat::load_from_reader(Cursor::new(include_bytes!($location))).unwrap();
                    Arc::new(RwLock::new(icon))
                };
            }

            pub fn [<get_ $name _icon>]() -> Arc<RwLock<TextureRGBAFloat>> {
                [<$name:upper>].clone()
            }

            pub fn [<get_ $name _icon_id>]() -> gl::types::GLuint {
                [<$name:upper>].write().unwrap().get_gl_tex()
            }
        }
    };
}

define_icon!(
    viewport_solid_shading,
    "../icons/viewport_solid_shading.png"
);
define_icon!(
    viewport_rendered_shading,
    "../icons/viewport_render_shading.png"
);
