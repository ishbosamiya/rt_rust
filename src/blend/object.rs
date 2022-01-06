use std::convert::{TryFrom, TryInto};

use blend::Instance;

use super::{
    id::{IDObject, ID},
    FromBlend, RotationModes,
};

#[derive(Debug)]
pub struct Object {
    id: ID,
    /// String describing subobject info, MAX_ID_NAME-2.
    parsubstr: String,
    /// Pointer to objects data - an 'ID' or NULL.
    ///
    /// This is not exactly the same as Blender, in Blender it is
    /// stored as a void pointer which isn't a good idea any
    /// language. Instead an Enum of all the possible objects that
    /// have an ID is stored.
    data: Option<IDObject>,
    loc: [f32; 3],
    rot: [f32; 3],
    /// Scale (can be negative).
    scale: [f32; 3],
    /// Final worldspace matrix with constraints & animsys applied.
    ///
    /// For simplicity reasons, the 4x4 matrix is stored in a single
    /// 16 element array. Stored column wise.
    obmat: [f32; 16],

    /// Rotation mode - uses defines set out in DNA_action_types.h for
    /// PoseChannel rotations....
    rotmode: RotationModes,
}

impl Object {
    /// Get a reference to the object's id.
    pub fn get_id(&self) -> &ID {
        &self.id
    }

    /// Get a reference to the object's data.
    pub fn get_data(&self) -> Option<&IDObject> {
        self.data.as_ref()
    }

    /// Get a reference to the object's loc.
    pub fn get_loc(&self) -> &[f32; 3] {
        &self.loc
    }

    /// Get a reference to the object's rot.
    pub fn get_rot(&self) -> &[f32; 3] {
        &self.rot
    }

    /// Get a reference to the object's scale.
    pub fn get_scale(&self) -> &[f32; 3] {
        &self.scale
    }

    /// Get a reference to the object's parsubstr.
    pub fn get_parsubstr(&self) -> &str {
        self.parsubstr.as_ref()
    }

    /// Get a reference to the object's obmat.
    pub fn get_obmat(&self) -> &[f32; 16] {
        &self.obmat
    }

    /// Get object's rotmode.
    pub fn get_rotmode(&self) -> RotationModes {
        self.rotmode
    }
}

impl FromBlend for Object {
    fn from_blend_instance(instance: &Instance) -> Option<Self> {
        if !instance.is_valid("data")
            || !instance.is_valid("loc")
            || !instance.is_valid("rot")
            // must use size and not scale, see struct member renaming
            // in module level docs
            || !instance.is_valid("size")
            || !instance.is_valid("obmat")
            || !instance.is_valid("rotmode")
        {
            println!("something not available");
            return None;
        }

        let loc = instance.get_f32_vec("loc");
        let rot = instance.get_f32_vec("rot");
        let scale = instance.get_f32_vec("size");

        let obmat = instance.get_f32_vec("obmat");

        Some(Self {
            id: ID::from_blend_instance(&instance.get("id"))?,
            parsubstr: instance.get_string("parsubstr"),
            data: IDObject::from_blend_instance(&instance.get("data")),
            loc: [loc[0], loc[1], loc[2]],
            rot: [rot[0], rot[1], rot[2]],
            scale: [scale[0], scale[1], scale[2]],
            obmat: [
                obmat[0], obmat[1], obmat[2], obmat[3], obmat[4], obmat[5], obmat[6], obmat[7],
                obmat[8], obmat[9], obmat[10], obmat[11], obmat[12], obmat[13], obmat[14],
                obmat[15],
            ],
            rotmode: instance.get_i16("rotmode").try_into().unwrap(),
        })
    }
}

impl TryFrom<i16> for RotationModes {
    type Error = ();

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(Self::AxisAngle),
            0 => Ok(Self::Quaternion),
            1 => Ok(Self::EulerXYZ),
            2 => Ok(Self::EulerXZY),
            3 => Ok(Self::EulerYXZ),
            4 => Ok(Self::EulerYZX),
            5 => Ok(Self::EulerZXY),
            6 => Ok(Self::EulerZYX),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use blend::Blend;

    use crate::blend::FromBlend;

    use super::Object;

    #[test]
    fn blend_object_test_01() {
        let cargo_manifest_dir = std::path::PathBuf::from(
            std::env::var_os("CARGO_MANIFEST_DIR").expect("could not find cargo manifest dir"),
        );
        let blend_path = cargo_manifest_dir.join("test.blend");
        let blend = Blend::from_path(blend_path);

        blend.get_by_code(*b"OB").iter().for_each(|instance| {
            let object = Object::from_blend_instance(instance);

            dbg!(&object);
        });
    }
}
