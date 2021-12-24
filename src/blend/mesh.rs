use std::convert::TryInto;

use blend::Instance;

use super::{id::ID, FromBlend};

/// Mesh Faces
///
/// This only stores the polygon size & flags, the vertex & edge
/// indices are stored in the #MLoop.
///
/// Typically accessed from #Mesh.mpoly.
#[derive(Debug)]
pub struct MPoly {
    /// Offset into loop array and number of loops in the face.
    loopstart: i32,
    /// Keep signed since we need to subtract when getting the
    /// previous loop.
    totloop: i32,
    mat_nr: i16,
    flag: i8,
    _pad: i8,
}

impl MPoly {
    /// Get mpoly's loopstart.
    pub fn get_loopstart(&self) -> i32 {
        self.loopstart
    }

    /// Get mpoly's totloop.
    pub fn get_totloop(&self) -> i32 {
        self.totloop
    }

    /// Get mpoly's mat nr.
    pub fn get_mat_nr(&self) -> i16 {
        self.mat_nr
    }

    /// Get mpoly's flag.
    pub fn get_flag(&self) -> i8 {
        self.flag
    }
}

// TODO: MPolyFlag

impl FromBlend for MPoly {
    fn from_blend_instance(instance: &Instance) -> Option<Self> {
        if !instance.is_valid("loopstart")
            || !instance.is_valid("totloop")
            || !instance.is_valid("mat_nr")
            || !instance.is_valid("flag")
            || !instance.is_valid("_pad")
        {
            return None;
        }

        Some(Self {
            loopstart: instance.get_i32("loopstart"),
            totloop: instance.get_i32("totloop"),
            mat_nr: instance.get_i16("mat_nr"),
            flag: instance.get_i8("flag"),
            _pad: instance.get_i8("_pad"),
        })
    }
}

/// Mesh Loops.
///
/// Each loop represents the corner of a polygon (#MPoly).
///
/// Typically accessed from #Mesh.mloop.
#[derive(Debug)]
pub struct MLoop {
    /// Vertex index.
    v: u32,
    /// Edge index.
    ///
    /// The e here is because we want to move away from relying on edge hashes.
    e: u32,
}

impl MLoop {
    /// Get mloop's v.
    pub fn get_v(&self) -> u32 {
        self.v
    }

    /// Get mloop's e.
    pub fn get_e(&self) -> u32 {
        self.e
    }
}

impl FromBlend for MLoop {
    fn from_blend_instance(instance: &Instance) -> Option<Self> {
        if !instance.is_valid("v") || !instance.is_valid("e") {
            return None;
        }

        Some(Self {
            // cannot get u32 directly since .blend files don't
            // support u32, in the file, it is stored as i32
            v: instance.get_i32("v").try_into().unwrap(),
            e: instance.get_i32("e").try_into().unwrap(),
        })
    }
}

/// UV coordinate for a polygon face & flag for selection & other
/// options.
#[derive(Debug)]
pub struct MLoopUV {
    uv: [f32; 2],
    flag: i32,
}

impl MLoopUV {
    /// Get mloop uv's uv.
    pub fn get_uv(&self) -> &[f32; 2] {
        &self.uv
    }

    /// Get mloop uv's flag.
    pub fn get_flag(&self) -> i32 {
        self.flag
    }
}

// TODO: MLoopUVFlag

impl FromBlend for MLoopUV {
    fn from_blend_instance(instance: &Instance) -> Option<Self> {
        if !instance.is_valid("uv") || !instance.is_valid("flag") {
            return None;
        }

        let uv = instance.get_f32_vec("uv");
        assert_eq!(uv.len(), 2);

        Some(Self {
            uv: [uv[0], uv[1]],
            flag: instance.get_i32("flag"),
        })
    }
}

/// While alpha is not currently in the 3D Viewport, this may
/// eventually be added back, keep this value set to 255.
#[derive(Debug)]
pub struct MLoopCol {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl MLoopCol {
    /// Get mloop col's r.
    pub fn get_r(&self) -> u8 {
        self.r
    }

    /// Get mloop col's g.
    pub fn get_g(&self) -> u8 {
        self.g
    }

    /// Get mloop col's b.
    pub fn get_b(&self) -> u8 {
        self.b
    }

    /// Get mloop col's a.
    pub fn get_a(&self) -> u8 {
        self.a
    }
}

impl FromBlend for MLoopCol {
    fn from_blend_instance(instance: &Instance) -> Option<Self> {
        if !instance.is_valid("r")
            || !instance.is_valid("g")
            || !instance.is_valid("b")
            || !instance.is_valid("a")
        {
            return None;
        }

        Some(Self {
            r: instance.get_u8("r"),
            g: instance.get_u8("g"),
            b: instance.get_u8("b"),
            a: instance.get_u8("a"),
        })
    }
}

/// Mesh Vertices.
///
/// Typically accessed from #Mesh.mvert
#[derive(Debug)]
pub struct MVert {
    co: [f32; 3],
    /// Cache the normal, can always be recalculated from surrounding
    /// faces. See #CD_CUSTOMLOOPNORMAL for custom normals.
    no: [i16; 3],
    flag: i8,
    bweight: i8,
}

impl MVert {
    /// Get mvert's co.
    pub fn get_co(&self) -> &[f32; 3] {
        &self.co
    }

    /// Get mvert's no.
    pub fn get_no(&self) -> &[i16; 3] {
        &self.no
    }

    /// Get mvert's flag.
    pub fn get_flag(&self) -> i8 {
        self.flag
    }

    /// Get mvert's bweight.
    pub fn get_bweight(&self) -> i8 {
        self.bweight
    }
}

// TODO: MVertFlag

impl FromBlend for MVert {
    fn from_blend_instance(instance: &Instance) -> Option<Self> {
        if !instance.is_valid("co")
            || !instance.is_valid("no")
            || !instance.is_valid("flag")
            || !instance.is_valid("bweight")
        {
            return None;
        }

        let co = instance.get_f32_vec("co");
        assert_eq!(co.len(), 3);
        let no = instance.get_i16_vec("no");
        assert_eq!(no.len(), 3);

        Some(Self {
            co: [co[0], co[1], co[2]],
            no: [no[0], no[1], no[2]],
            flag: instance.get_i8("flag"),
            bweight: instance.get_i8("bweight"),
        })
    }
}

/// Mesh Edges.
///
/// Typically accessed from #Mesh.medge
#[derive(Debug)]
pub struct MEdge {
    /// Un-ordered vertex indices (cannot match).
    v1: u32,
    v2: u32,
    crease: i8,
    bweight: i8,
    flag: i16,
}

impl MEdge {
    /// Get medge's v1.
    pub fn get_v1(&self) -> u32 {
        self.v1
    }

    /// Get medge's v2.
    pub fn get_v2(&self) -> u32 {
        self.v2
    }

    /// Get medge's crease.
    pub fn get_crease(&self) -> i8 {
        self.crease
    }

    /// Get medge's bweight.
    pub fn get_bweight(&self) -> i8 {
        self.bweight
    }

    /// Get medge's flag.
    pub fn get_flag(&self) -> i16 {
        self.flag
    }
}

// TODO: MEdgeFlag

impl FromBlend for MEdge {
    fn from_blend_instance(instance: &Instance) -> Option<Self> {
        if !instance.is_valid("v1")
            || !instance.is_valid("v2")
            || !instance.is_valid("crease")
            || !instance.is_valid("bweight")
            || !instance.is_valid("flag")
        {
            return None;
        }

        Some(Self {
            // cannot get u32 directly since .blend files don't
            // support u32, in the file, it is stored as i32
            v1: instance.get_i32("v1").try_into().unwrap(),
            v2: instance.get_i32("v2").try_into().unwrap(),
            crease: instance.get_i8("crease"),
            bweight: instance.get_i8("bweight"),
            flag: instance.get_i16("flag"),
        })
    }
}

#[derive(Debug)]
pub struct Mesh {
    id: ID,
    mpoly: Vec<MPoly>,
    mloop: Vec<MLoop>,
    mloopuv: Vec<MLoopUV>,
    mloopcol: Vec<MLoopCol>,
    mvert: Vec<MVert>,
    medge: Vec<MEdge>,
}

impl Mesh {
    /// Get a reference to the mesh's id.
    pub fn get_id(&self) -> &ID {
        &self.id
    }

    /// Get a reference to the mesh's mpoly.
    pub fn get_mpoly(&self) -> &[MPoly] {
        self.mpoly.as_ref()
    }

    /// Get a reference to the mesh's mloop.
    pub fn get_mloop(&self) -> &[MLoop] {
        self.mloop.as_ref()
    }

    /// Get a reference to the mesh's mloopuv.
    pub fn get_mloopuv(&self) -> &[MLoopUV] {
        self.mloopuv.as_ref()
    }

    /// Get a reference to the mesh's mloopcol.
    pub fn get_mloopcol(&self) -> &[MLoopCol] {
        self.mloopcol.as_ref()
    }

    /// Get a reference to the mesh's mvert.
    pub fn get_mvert(&self) -> &[MVert] {
        self.mvert.as_ref()
    }

    /// Get a reference to the mesh's medge.
    pub fn get_medge(&self) -> &[MEdge] {
        self.medge.as_ref()
    }
}

impl FromBlend for Mesh {
    fn from_blend_instance(instance: &Instance) -> Option<Self> {
        if !instance.is_valid("id")
            || !instance.is_valid("totvert")
            || !instance.is_valid("totedge")
            || !instance.is_valid("totpoly")
            || !instance.is_valid("totloop")
        // cannot check for mpoly, mloop, mloopuv, mloopcol, mvert,
        // medge, etc. here since they may or may not exist, it is
        // possible for all of them to be empty
        {
            return None;
        }

        let totvert: usize = instance.get_i32("totvert").try_into().unwrap();
        let totedge: usize = instance.get_i32("totedge").try_into().unwrap();
        let totpoly: usize = instance.get_i32("totpoly").try_into().unwrap();
        let totloop: usize = instance.get_i32("totloop").try_into().unwrap();

        let mpoly: Vec<MPoly> = if instance.is_valid("mpoly") {
            instance
                .get_iter("mpoly")
                .map(|instance| MPoly::from_blend_instance(&instance).unwrap())
                .collect()
        } else {
            vec![]
        };
        let mloop: Vec<MLoop> = if instance.is_valid("mloop") {
            instance
                .get_iter("mloop")
                .map(|instance| MLoop::from_blend_instance(&instance).unwrap())
                .collect()
        } else {
            vec![]
        };
        let mloopuv: Vec<MLoopUV> = if instance.is_valid("mloopuv") {
            instance
                .get_iter("mloopuv")
                .map(|instance| MLoopUV::from_blend_instance(&instance).unwrap())
                .collect()
        } else {
            vec![]
        };
        let mloopcol: Vec<MLoopCol> = if instance.is_valid("mloopcol") {
            instance
                .get_iter("mloopcol")
                .map(|instance| MLoopCol::from_blend_instance(&instance).unwrap())
                .collect()
        } else {
            vec![]
        };
        let mvert: Vec<MVert> = if instance.is_valid("mvert") {
            instance
                .get_iter("mvert")
                .map(|instance| MVert::from_blend_instance(&instance).unwrap())
                .collect()
        } else {
            vec![]
        };
        let medge: Vec<MEdge> = if instance.is_valid("medge") {
            instance
                .get_iter("medge")
                .map(|instance| MEdge::from_blend_instance(&instance).unwrap())
                .collect()
        } else {
            vec![]
        };

        assert_eq!(mpoly.len(), totpoly);
        assert_eq!(mloop.len(), totloop);
        assert_eq!(mvert.len(), totvert);
        assert_eq!(medge.len(), totedge);

        Some(Self {
            id: ID::from_blend_instance(&instance.get("id")).unwrap(),
            mpoly,
            mloop,
            mloopuv,
            mloopcol,
            mvert,
            medge,
        })
    }
}
