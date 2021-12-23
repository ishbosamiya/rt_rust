use std::collections::HashMap;

use lazy_static::lazy_static;

use blend::Instance;

use super::{mesh::Mesh, FromBlend};

lazy_static! {
    static ref ID_NAME_MAP: HashMap<&'static [u8; 4], &'static str> = {
        let mut map = HashMap::new();
        map.insert(b"SC\0\0", "Scene");
        map.insert(b"LI\0\0", "Library");
        map.insert(b"OB\0\0", "Object");
        map.insert(b"ME\0\0", "Mesh");
        map.insert(b"CU\0\0", "Curve");
        map.insert(b"MB\0\0", "MetaBall");
        map.insert(b"MA\0\0", "Material");
        map.insert(b"TE\0\0", "Tex (Texture)");
        map.insert(b"IM\0\0", "Image");
        map.insert(b"LT\0\0", "Lattice");
        map.insert(b"LA\0\0", "Light");
        map.insert(b"CA\0\0", "Camera");
        map.insert(b"IP\0\0", "Ipo (depreciated, replaced by FCurves)");
        map.insert(b"KE\0\0", "Key (shape key)");
        map.insert(b"WO\0\0", "World");
        map.insert(b"SR\0\0", "Screen");
        map.insert(b"VF\0\0", "VFont (Vector Font)");
        map.insert(b"TX\0\0", "Text");
        map.insert(b"SK\0\0", "Speaker");
        map.insert(b"SO\0\0", "Sound");
        map.insert(b"GR\0\0", "Collection");
        map.insert(b"AR\0\0", "bArmature");
        map.insert(b"AC\0\0", "bAction");
        map.insert(b"NT\0\0", "bNodeTree");
        map.insert(b"BR\0\0", "Brush");
        map.insert(b"PA\0\0", "ParticleSettings");
        map.insert(b"GD\0\0", "bGPdata, (Grease Pencil)");
        map.insert(b"WM\0\0", "WindowManager");
        map.insert(b"MC\0\0", "MovieClip");
        map.insert(b"MS\0\0", "Mask");
        map.insert(b"LS\0\0", "FreestyleLineStyle");
        map.insert(b"PL\0\0", "Palette");
        map.insert(b"PC\0\0", "PaintCurve");
        map.insert(b"CF\0\0", "CacheFile");
        map.insert(b"WS\0\0", "WorkSpace");
        map.insert(b"LP\0\0", "LightProbe");
        map.insert(b"HA\0\0", "Hair");
        map.insert(b"PT\0\0", "PointCloud");
        map.insert(b"VO\0\0", "Volume");
        map.insert(b"SI\0\0", "Simulation (geometry node groups)");
        map
    };
}

/// IDs from Blender's `DNA_ID_enums.h`
///
/// ```
/// /**
///  * ID from database.
///  *
///  * Written to #BHead.code (for file IO)
///  * and the first 2 bytes of #ID.name (for runtime checks, see #GS macro).
///  */
/// typedef enum ID_Type {
///   ID_SCE = MAKE_ID2('S', 'C'), /* Scene */
///   ID_LI = MAKE_ID2('L', 'I'),  /* Library */
///   ID_OB = MAKE_ID2('O', 'B'),  /* Object */
///   ID_ME = MAKE_ID2('M', 'E'),  /* Mesh */
///   ID_CU = MAKE_ID2('C', 'U'),  /* Curve */
///   ID_MB = MAKE_ID2('M', 'B'),  /* MetaBall */
///   ID_MA = MAKE_ID2('M', 'A'),  /* Material */
///   ID_TE = MAKE_ID2('T', 'E'),  /* Tex (Texture) */
///   ID_IM = MAKE_ID2('I', 'M'),  /* Image */
///   ID_LT = MAKE_ID2('L', 'T'),  /* Lattice */
///   ID_LA = MAKE_ID2('L', 'A'),  /* Light */
///   ID_CA = MAKE_ID2('C', 'A'),  /* Camera */
///   ID_IP = MAKE_ID2('I', 'P'),  /* Ipo (depreciated, replaced by FCurves) */
///   ID_KE = MAKE_ID2('K', 'E'),  /* Key (shape key) */
///   ID_WO = MAKE_ID2('W', 'O'),  /* World */
///   ID_SCR = MAKE_ID2('S', 'R'), /* Screen */
///   ID_VF = MAKE_ID2('V', 'F'),  /* VFont (Vector Font) */
///   ID_TXT = MAKE_ID2('T', 'X'), /* Text */
///   ID_SPK = MAKE_ID2('S', 'K'), /* Speaker */
///   ID_SO = MAKE_ID2('S', 'O'),  /* Sound */
///   ID_GR = MAKE_ID2('G', 'R'),  /* Collection */
///   ID_AR = MAKE_ID2('A', 'R'),  /* bArmature */
///   ID_AC = MAKE_ID2('A', 'C'),  /* bAction */
///   ID_NT = MAKE_ID2('N', 'T'),  /* bNodeTree */
///   ID_BR = MAKE_ID2('B', 'R'),  /* Brush */
///   ID_PA = MAKE_ID2('P', 'A'),  /* ParticleSettings */
///   ID_GD = MAKE_ID2('G', 'D'),  /* bGPdata, (Grease Pencil) */
///   ID_WM = MAKE_ID2('W', 'M'),  /* WindowManager */
///   ID_MC = MAKE_ID2('M', 'C'),  /* MovieClip */
///   ID_MSK = MAKE_ID2('M', 'S'), /* Mask */
///   ID_LS = MAKE_ID2('L', 'S'),  /* FreestyleLineStyle */
///   ID_PAL = MAKE_ID2('P', 'L'), /* Palette */
///   ID_PC = MAKE_ID2('P', 'C'),  /* PaintCurve */
///   ID_CF = MAKE_ID2('C', 'F'),  /* CacheFile */
///   ID_WS = MAKE_ID2('W', 'S'),  /* WorkSpace */
///   ID_LP = MAKE_ID2('L', 'P'),  /* LightProbe */
///   ID_HA = MAKE_ID2('H', 'A'),  /* Hair */
///   ID_PT = MAKE_ID2('P', 'T'),  /* PointCloud */
///   ID_VO = MAKE_ID2('V', 'O'),  /* Volume */
///   ID_SIM = MAKE_ID2('S', 'I'), /* Simulation (geometry node groups) */
/// } ID_Type;
/// ```
#[derive(Debug)]
pub struct ID {
    /// First 2 characters are for the ID code followed by the actual
    /// name
    name: String,
}

impl ID {
    /// Get a reference to the id's name.
    pub fn get_name(&self) -> &str {
        self.name.as_ref()
    }
}

impl FromBlend for ID {
    fn from_blend_instance(instance: &Instance) -> Option<Self> {
        if !instance.is_valid("name") {
            return None;
        }

        Some(Self {
            name: instance.get_string("name"),
        })
    }
}

/// This is not in Blender but needed to mimic similar behavior. All
/// objects have their data (mesh/curve/metaball/etc.) stored in a
/// void pointer. This data generally has an ID associated with it,
/// the interface created in Blender is implicit due to C limitations
/// and legacy code. Here similar thing is mimiced using an
/// enum. Anything that stores an ID must be added here.
#[derive(Debug)]
pub enum IDObject {
    Mesh(Mesh),
}

impl FromBlend for IDObject {
    fn from_blend_instance(instance: &Instance) -> Option<Self> {
        if instance.code()[0..=1] == *b"ME" {
            Some(Self::Mesh(Mesh::from_blend_instance(instance)?))
        } else {
            eprintln!(
                "TODO: Need to implement for id: {} code: {:?}",
                ID_NAME_MAP.get(&instance.code()).unwrap(),
                instance.code()
            );
            None
        }
    }
}
