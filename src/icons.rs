use lazy_static::lazy_static;
use paste::paste;
use quick_renderer::texture::TextureRGBAFloat;

use std::{
    io::Cursor,
    sync::{Arc, RwLock},
};

macro_rules! define_icon {
    ( $name:ident , $location:tt ) => {
        paste! {
            lazy_static! {
                static ref  [<$name:upper>] : Arc<RwLock<TextureRGBAFloat>> = {
                    let icon = TextureRGBAFloat::load_from_reader(Cursor::new(include_bytes!($location))).unwrap();
                    Arc::new(RwLock::new(icon))
                };
            }

            #[doc = "Get texture of the icon " $name:upper]
            ///
            /// # Safety
            ///
            #[doc = "It is advisable to get the (OpenGL) id using [`" [<get_ $name _icon_id>] "`] which is generally what is required. Making the access to the texture itself harder is to ensure unnecessary deadlocks do not occur and to encourage the user to use the (OpenGL) id instead."]
            pub unsafe fn [<get_ $name _icon>]() -> Arc<RwLock<TextureRGBAFloat>> {
                [<$name:upper>].clone()
            }

            #[doc = "Get (OpenGL) id of the texture of the icon " $name:upper]
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
