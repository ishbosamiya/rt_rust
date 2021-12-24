use ordered_float::OrderedFloat;

use crate::{blend, glm, util};

use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct MeshIO {
    pub positions: Vec<glm::DVec3>,
    pub uvs: Vec<glm::DVec2>,
    pub normals: Vec<glm::DVec3>,
    pub face_indices: Vec<Vec<(usize, usize, usize)>>,
    pub face_has_uv: bool,
    pub face_has_normal: bool,
    pub line_indices: Vec<Vec<usize>>,

    /// end of object indices stored for (positions, uvs, normals,
    /// face_indices, line_indices)
    pub end_of_object: Vec<(usize, usize, usize, usize, usize)>,
    /// object names, if no name available, set to None
    pub object_names: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeshIOError {
    Io(String),
    InvalidFile,
    Unknown,
}

impl From<std::io::Error> for MeshIOError {
    fn from(err: std::io::Error) -> MeshIOError {
        MeshIOError::Io(err.to_string())
    }
}

impl std::fmt::Display for MeshIOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshIOError::Io(error) => write!(f, "io error {}", error),
            MeshIOError::InvalidFile => write!(f, "invalid file"),
            MeshIOError::Unknown => write!(f, "unknown error"),
        }
    }
}

impl std::error::Error for MeshIOError {}

impl MeshIO {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            uvs: Vec::new(),
            normals: Vec::new(),
            face_indices: Vec::new(),
            face_has_uv: false,
            face_has_normal: false,
            line_indices: Vec::new(),
            end_of_object: Vec::new(),
            object_names: Vec::new(),
        }
    }

    pub fn read<P>(path: P) -> Result<Self, MeshIOError>
    where
        P: AsRef<Path>,
    {
        match path.as_ref().extension() {
            Some(extension) => match extension.to_str().unwrap() {
                "obj" => Self::read_obj(path.as_ref()),
                "blend" => Self::read_blend_from_path(path),
                _ => Err(MeshIOError::Unknown),
            },
            None => Err(MeshIOError::Unknown),
        }
    }

    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), MeshIOError> {
        match path.as_ref().extension() {
            Some(extension) => match extension.to_str().unwrap() {
                "obj" => self.write_obj(path),
                _ => Err(MeshIOError::Unknown),
            },
            None => Err(MeshIOError::Unknown),
        }
    }

    pub fn from_lines(lines: &[&str]) -> Result<Self, MeshIOError> {
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut normals = Vec::new();
        let mut face_indices = Vec::new();
        let mut face_has_uv = false;
        let mut face_has_normal = false;
        let mut line_indices = Vec::new();
        let mut end_of_object = Vec::new();
        let mut object_names = Vec::new();

        for line in lines {
            Self::process_line(
                line,
                &mut positions,
                &mut uvs,
                &mut normals,
                &mut face_indices,
                &mut face_has_uv,
                &mut face_has_normal,
                &mut line_indices,
                &mut end_of_object,
                &mut object_names,
            )?
        }

        // if there is only one object and it wasn't assigned a name,
        // push None to object_names so that indexing remains correct
        if object_names.is_empty() {
            object_names.push(None);
        }

        Ok(MeshIO {
            positions,
            uvs,
            normals,
            face_indices,
            face_has_uv,
            face_has_normal,
            line_indices,
            end_of_object,
            object_names,
        })
    }

    /// Splits the meshio into the constituent objects
    pub fn split(mut self) -> Vec<Self> {
        // End of object for the last object may not be specified but
        // it's name should exist (Some(name) or None to indicate the
        // number of objects. Update end of object list if needed.
        if self.end_of_object.len() == self.object_names.len() - 1 {
            self.end_of_object.push((
                self.positions.len(),
                self.uvs.len(),
                self.normals.len(),
                self.face_indices.len(),
                self.line_indices.len(),
            ));
        }

        assert_eq!(self.end_of_object.len(), self.object_names.len());

        let mut prev_position = 0;
        let mut prev_uv = 0;
        let mut prev_normal = 0;
        let mut prev_face = 0;
        let mut prev_line = 0;
        self.end_of_object
            .iter()
            .enumerate()
            .map(
                |(object_index, (end_position, end_uv, end_normal, end_face, end_line))| {
                    let positions = self.positions[prev_position..*end_position].to_vec();
                    let uvs = self.uvs[prev_uv..*end_uv].to_vec();
                    let normals = self.normals[prev_normal..*end_normal].to_vec();
                    let face_indices = self.face_indices[prev_face..*end_face]
                        .iter()
                        .map(|face| {
                            face.iter()
                                .map(|(pos, uv, normal)| {
                                    (pos - prev_position, uv - prev_uv, normal - prev_normal)
                                })
                                .collect()
                        })
                        .collect();
                    let line_indices = self.line_indices[prev_line..*end_line]
                        .iter()
                        .map(|line| line.iter().map(|pos| pos - prev_position).collect())
                        .collect();

                    prev_position = *end_position;
                    prev_uv = *end_uv;
                    prev_normal = *end_normal;
                    prev_face = *end_face;
                    prev_line = *end_line;

                    Self {
                        positions,
                        uvs,
                        normals,
                        face_indices,
                        face_has_uv: self.face_has_uv,
                        face_has_normal: self.face_has_normal,
                        line_indices,
                        end_of_object: Vec::new(),
                        object_names: vec![self.object_names[object_index].clone()],
                    }
                },
            )
            .collect()
    }

    fn read_obj(path: &Path) -> Result<MeshIO, MeshIOError> {
        let fin = File::open(path)?;
        let reader = BufReader::new(fin);

        Self::from_lines(
            &reader
                .lines()
                .map(|line| line.unwrap())
                .collect::<Vec<_>>()
                .iter()
                .map(|line| line.as_str())
                .collect::<Vec<_>>(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn process_line(
        line: &str,
        positions: &mut Vec<glm::DVec3>,
        uvs: &mut Vec<glm::DVec2>,
        normals: &mut Vec<glm::DVec3>,
        face_indices: &mut Vec<Vec<(usize, usize, usize)>>,
        face_has_uv: &mut bool,
        face_has_normal: &mut bool,
        line_indices: &mut Vec<Vec<usize>>,
        end_of_object: &mut Vec<(usize, usize, usize, usize, usize)>,
        object_names: &mut Vec<Option<String>>,
    ) -> Result<(), MeshIOError> {
        if line.starts_with('#') {
            return Ok(());
        }
        let vals: Vec<&str> = line.split(' ').collect();

        assert!(!vals.is_empty());
        match vals[0] {
            "v" => {
                // Don't currently support positions with 4 or more coordinates
                assert!(vals.len() == 4);
                let x: f64 = vals[1].parse().unwrap();
                let y: f64 = vals[2].parse().unwrap();
                let z: f64 = vals[3].parse().unwrap();
                positions.push(glm::vec3(x, y, z));
                Ok(())
            }
            "vn" => {
                // Don't currently support positions with 4 or more coordinates
                assert!(vals.len() == 4);
                let x: f64 = vals[1].parse().unwrap();
                let y: f64 = vals[2].parse().unwrap();
                let z: f64 = vals[3].parse().unwrap();
                normals.push(glm::vec3(x, y, z));
                Ok(())
            }
            "vt" => {
                // Don't currently support texture coordinates with 3 or more coordinates
                assert!(vals.len() == 3);
                let u: f64 = vals[1].parse().unwrap();
                let v: f64 = vals[2].parse().unwrap();
                uvs.push(glm::vec2(u, v));
                Ok(())
            }
            "f" => {
                // Don't currently support face with 2 or lesser verts
                assert!(vals.len() >= 4);
                let mut face_i: Vec<(usize, usize, usize)> = Vec::new();
                for val in vals.iter().skip(1) {
                    let indices: Vec<&str> = val.split('/').collect();
                    match indices.len() {
                        // only positions
                        1 => {
                            let pos_index: usize = indices[0].parse().unwrap();
                            face_i.push((pos_index - 1, usize::MAX, usize::MAX));
                        }
                        // positions and texture coordinates
                        2 => {
                            let pos_index: usize = indices[0].parse().unwrap();
                            let uv_index: usize = indices[1].parse().unwrap();
                            face_i.push((pos_index - 1, uv_index - 1, usize::MAX));
                            *face_has_uv = true;
                        }
                        // positions, texture coordinates and normals
                        3 => {
                            let pos_index: usize = indices[0].parse().unwrap();
                            let uv_index: usize;
                            if !indices[1].is_empty() {
                                uv_index = indices[1].parse().unwrap();
                            } else {
                                uv_index = usize::MAX;
                            }
                            let normal_index: usize = indices[2].parse().unwrap();
                            if uv_index == usize::MAX {
                                face_i.push((pos_index - 1, uv_index, normal_index - 1));
                            } else {
                                face_i.push((pos_index - 1, uv_index - 1, normal_index - 1));
                            }
                            *face_has_uv = true;
                            *face_has_normal = true;
                        }
                        _ => {
                            return Err(MeshIOError::InvalidFile);
                        }
                    }
                }
                assert!(!face_i.is_empty());
                face_indices.push(face_i);
                Ok(())
            }
            "l" => {
                assert!(vals.len() >= 3);
                let mut indices: Vec<usize> = Vec::new();
                for val in vals.iter().skip(1) {
                    let index: usize = val.parse().unwrap();
                    indices.push(index - 1);
                }
                line_indices.push(indices);
                Ok(())
            }
            "o" => {
                assert!(vals.len() >= 2);
                if positions.is_empty()
                    && uvs.is_empty()
                    && normals.is_empty()
                    && face_indices.is_empty()
                    && line_indices.is_empty()
                {
                    // if nothing processed until now, no object could
                    // have been created so just skip
                } else {
                    // it is possible for the first object to not have
                    // a name assigned, at this point the second
                    // object starts so for proper indexing, None must
                    // be pushed to object_names if it is empty
                    if object_names.is_empty() {
                        object_names.push(None);
                    }
                    end_of_object.push((
                        positions.len(),
                        uvs.len(),
                        normals.len(),
                        face_indices.len(),
                        line_indices.len(),
                    ));
                }
                object_names.push(Some(vals[1].to_string()));
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn write_obj<P: AsRef<Path>>(&self, path: P) -> Result<(), MeshIOError> {
        let mut file = std::fs::File::create(path)?;
        self.positions
            .iter()
            .try_for_each(|pos| writeln!(file, "v {} {} {}", pos[0], pos[1], pos[2]))?;

        self.uvs
            .iter()
            .try_for_each(|uv| writeln!(file, "vt {} {}", uv[0], uv[1]))?;

        self.normals.iter().try_for_each(|normal| {
            writeln!(file, "vn {} {} {}", normal[0], normal[1], normal[2])
        })?;

        self.face_indices.iter().try_for_each(|face| {
            write!(file, "f")?;
            face.iter()
                .try_for_each(|(pos_index, uv_index, normal_index)| {
                    // TODO(ish): support uv index and normal index being invalid

                    write!(
                        file,
                        " {}/{}/{}",
                        pos_index + 1,
                        uv_index + 1,
                        normal_index + 1
                    )
                })?;
            writeln!(file)
        })?;

        self.line_indices.iter().try_for_each(|line| {
            write!(file, "l")?;
            line.iter()
                .try_for_each(|index| write!(file, " {}", index + 1))?;
            writeln!(file)
        })?;

        Ok(())
    }

    pub fn read_blend_from_path(path: impl AsRef<Path>) -> Result<MeshIO, MeshIOError> {
        let mut file = File::open(&path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let data = if data[0..7] != *b"BLENDER" {
            if util::file_magic_is_gzip(&data) {
                let mut zip_archive =
                    zip::read::ZipArchive::new(std::io::Cursor::new(data)).unwrap();
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
        };

        Self::read_blend(std::io::Cursor::new(data))
    }

    pub fn read_blend(data: impl Read) -> Result<MeshIO, MeshIOError> {
        Ok(blend::get_all_objects(data)
            .iter()
            .filter(|object| object.get_data().is_some())
            .filter(|object| matches!(object.get_data().unwrap(), blend::id::IDObject::Mesh(_)))
            .fold(MeshIO::new(), |mut meshio, object| {
                meshio
                    .object_names
                    .push(Some(object.get_id().get_name()[2..].to_string()));

                let blend::id::IDObject::Mesh(mesh) = object.get_data().unwrap();

                let start_pos_index = meshio.positions.len();
                let start_uv_index = meshio.uvs.len();

                meshio.positions.extend(
                    mesh.get_mvert()
                        .iter()
                        .map::<glm::DVec3, _>(|mvert| glm::convert(glm::make_vec3(mvert.get_co()))),
                );

                meshio.normals.extend(
                    mesh.get_mvert()
                        .iter()
                        .map(|mvert| util::normal_i16_slice_to_dvec3(mvert.get_no())),
                );

                {
                    let mut true_uv_index = 0;
                    let mut uv_map: HashMap<(usize, [OrderedFloat<f32>; 2]), usize> =
                        HashMap::new();
                    mesh.get_mpoly().iter().for_each(|mpoly| {
                        let loopstart: usize = mpoly.get_loopstart().try_into().unwrap();
                        let totloop: usize = mpoly.get_totloop().try_into().unwrap();
                        let mut face = Vec::with_capacity(totloop);
                        (0..totloop).for_each(|j| {
                            let mloop: &blend::mesh::MLoop = &mesh.get_mloop()[loopstart + j];
                            let mloopuv: &blend::mesh::MLoopUV = &mesh.get_mloopuv()[loopstart + j];
                            let mloop_v = mloop.get_v().try_into().unwrap();
                            let pos_index: usize = mloop_v + start_pos_index;
                            let normal_index = pos_index;

                            let uv_ordered = [
                                OrderedFloat(mloopuv.get_uv()[0]),
                                OrderedFloat(mloopuv.get_uv()[1]),
                            ];
                            let uv_index =
                                *uv_map.entry((mloop_v, uv_ordered)).or_insert_with(|| {
                                    meshio
                                        .uvs
                                        .push(glm::convert(glm::make_vec2(mloopuv.get_uv())));

                                    true_uv_index += 1;

                                    true_uv_index - 1
                                }) + start_uv_index;

                            face.push((pos_index, uv_index, normal_index));
                        });

                        meshio.face_indices.push(face);
                    });
                }

                // TODO: add line indices support, currently not
                // possible since MEdgeFlags are not implemented

                meshio.end_of_object.push((
                    meshio.positions.len(),
                    meshio.uvs.len(),
                    meshio.normals.len(),
                    meshio.face_indices.len(),
                    meshio.line_indices.len(),
                ));
                meshio
            }))
    }
}

impl Default for MeshIO {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meshreader_read_obj_test_01() {
        let data = MeshIO::read_obj(Path::new("tests/obj_test_01.obj")).unwrap();
        assert_eq!(data.positions.len(), 5);
        assert_eq!(data.uvs.len(), 6);
        assert_eq!(data.normals.len(), 2);
        assert_eq!(data.face_indices.len(), 2);
        assert_eq!(data.face_indices[0].len(), 3);
        assert_eq!(data.positions[0], glm::vec3(0.778921, 1.572047, -0.878382));
        assert_eq!(data.line_indices.len(), 1);
        assert_eq!(data.line_indices[0].len(), 2);
    }

    #[test]
    fn meshreader_read_obj_test_02() {
        match MeshIO::read_obj(Path::new("tests/obj_test_02.obj")) {
            Err(error) => match error {
                MeshIOError::InvalidFile => (),
                _ => panic!("Should have gotten an invalid file error"),
            },
            Ok(_) => panic!("Should have gotten an invalid file error"),
        }
    }

    #[test]
    fn meshreader_read_obj_test_03() {
        MeshIO::read_obj(Path::new("tests/obj_test_03.obj")).unwrap();
    }

    #[test]
    fn meshreader_read_obj_test_04() {
        let meshio = MeshIO::read_obj(Path::new("tests/obj_test_07.obj")).unwrap();
        assert_eq!(meshio.end_of_object.len(), 1);
        assert_eq!(meshio.end_of_object[0], (8, 14, 6, 6, 0));
    }

    #[test]
    fn meshreader_read_obj_test_05() {
        let meshio = MeshIO::read_obj(Path::new("tests/obj_test_07.obj")).unwrap();
        let meshios = meshio.split();
        assert_eq!(meshios.len(), 2);
        let data = &meshios[0];
        assert_eq!(data.positions.len(), 8);
        assert_eq!(data.uvs.len(), 14);
        assert_eq!(data.normals.len(), 6);
        assert_eq!(data.face_indices.len(), 6);
        assert_eq!(data.face_indices[0].len(), 4);
        assert_eq!(data.line_indices.len(), 0);

        let data = &meshios[1];
        assert_eq!(data.positions.len(), 8);
        assert_eq!(data.uvs.len(), 14);
        assert_eq!(data.normals.len(), 6);
        assert_eq!(data.face_indices.len(), 6);
        assert_eq!(data.face_indices[0].len(), 4);
        assert_eq!(data.line_indices.len(), 0);
    }

    #[test]
    fn meshreader_read_blend_test_01() {
        let meshio = MeshIO::read("tests/blend_test_01.blend").unwrap();
        assert_eq!(meshio.positions.len(), 8);
        assert_eq!(meshio.uvs.len(), 14);
        assert_eq!(meshio.normals.len(), 8);
        assert_eq!(meshio.face_indices.len(), 6);
        assert_eq!(meshio.face_indices[0].len(), 4);
        assert_eq!(meshio.line_indices.len(), 0);
    }

    #[test]
    fn meshreader_read_blend_test_02() {
        let meshio = MeshIO::read("tests/blend_test_01.blend").unwrap();
        let meshios = meshio.split();
        assert_eq!(meshios.len(), 1);
        let meshio = &meshios[0];
        assert_eq!(meshio.positions.len(), 8);
        assert_eq!(meshio.uvs.len(), 14);
        assert_eq!(meshio.normals.len(), 8);
        assert_eq!(meshio.face_indices.len(), 6);
        assert_eq!(meshio.face_indices[0].len(), 4);
        assert_eq!(meshio.line_indices.len(), 0);
    }

    #[test]
    fn meshreader_read_blend_test_03() {
        let meshio = MeshIO::read("tests/blend_test_02.blend").unwrap();
        let meshios = meshio.split();
        assert_eq!(meshios.len(), 2);
        let meshio = meshios
            .iter()
            .find(|meshio| meshio.object_names[0].as_ref().unwrap() == "Cube")
            .unwrap();
        assert_eq!(meshio.positions.len(), 8);
        assert_eq!(meshio.uvs.len(), 14);
        assert_eq!(meshio.normals.len(), 8);
        assert_eq!(meshio.face_indices.len(), 6);
        assert_eq!(meshio.face_indices[0].len(), 4);
        assert_eq!(meshio.line_indices.len(), 0);

        let meshio = meshios
            .iter()
            .find(|meshio| meshio.object_names[0].as_ref().unwrap() == "Suzanne")
            .unwrap();
        assert_eq!(meshio.positions.len(), 507);
        assert_eq!(meshio.uvs.len(), 556);
        assert_eq!(meshio.normals.len(), 507);
        assert_eq!(meshio.face_indices.len(), 500);
        assert_eq!(meshio.line_indices.len(), 0);
    }
}
