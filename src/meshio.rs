use itertools::enumerate;

use crate::glm;

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct MeshIO {
    pub positions: Vec<glm::DVec3>,
    pub uvs: Vec<glm::DVec2>,
    pub normals: Vec<glm::DVec3>,
    pub face_indices: Vec<Vec<(usize, usize, usize)>>,
    pub face_has_uv: bool,
    pub face_has_normal: bool,
    pub line_indices: Vec<Vec<usize>>,
    pub face_starting: Vec<usize>,
    pub end_of_indices: Vec<(usize, usize, usize)>,
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
            face_starting: Vec::new(),
            end_of_indices: Vec::new(),
        }
    }

    pub fn split_mesh(self) {
        // Split each object into separate mesh io
        // Done by using face_starting and end_of_indices
        let meshes: Vec<Self>;

        let mut index = 0;
        let mut start_pos1; 
        let mut start_pos2;
        let mut start_pos3;
        for obj in self.face_indices {
            

            // Make new MeshIO Object
            // Add all types of face indeces and rest to it
            let (end_pos1, end_pos2, end_pos3) = self.end_of_indices[index];
            if index == 0 {
                start_pos1 = 0;
                start_pos2 = 0;
                start_pos3 = 0;
            }

            let pos_new = &self.positions[start_pos1..end_pos1];
            let uvs_new = &self.uvs[start_pos2..end_pos2];
            let normal_new = &self.normals[start_pos3..end_pos3];
            meshes.add(Self {
                positions: pos_new.to_vec(),
                uvs: uvs_new.to_vec(),
                normals: normal_new.to_vec(),
                face_indices: obj,
                face_has_uv: self.face_has_uv,
                face_has_normal: self.face_has_normal,
                line_indices: self.line_indices,
                face_starting: Vec::new(),
                end_of_indices: Vec::new(),
            });
            // TODO Append new values into new meshes vector
            start_pos1 = end_pos1;
            start_pos2 = end_pos2;
            start_pos3 = end_pos3;
            index += 1
        }
    }

    pub fn read(path: &Path) -> Result<Self, MeshIOError> {
        match path.extension() {
            Some(extension) => match extension.to_str().unwrap() {
                "obj" => Self::read_obj(path),
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
        let mut face_starting = Vec::new();
        let mut end_of_indices = Vec::new();

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
                &mut face_starting,
                &mut end_of_indices,
            )?
        }

        Ok(MeshIO {
            positions,
            uvs,
            normals,
            face_indices,
            face_has_uv,
            face_has_normal,
            line_indices,
            face_starting,
            end_of_indices,
        })
    }

    fn read_obj(path: &Path) -> Result<MeshIO, MeshIOError> {
        let fin = File::open(path)?;
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut normals = Vec::new();
        let mut face_indices = Vec::new();
        let mut face_has_uv = false;
        let mut face_has_normal = false;
        let mut line_indices = Vec::new();
        let mut face_starting = vec![0];
        let mut end_of_indices = vec![(0, 0, 0)];
        

        let mut file_data = std::fs::File::open(path).unwrap();
        let mut contents = String::new();
        file_data.read_to_string(&mut contents).unwrap();

        let reader = BufReader::new(fin);

        for line in reader.lines() {
            Self::process_line(
                &line?,
                &mut positions,
                &mut uvs,
                &mut normals,
                &mut face_indices,
                &mut face_has_uv,
                &mut face_has_normal,
                &mut line_indices,
                &mut face_starting,
                &mut end_of_indices,
            )?
        }

        // TODO(ish): validate the indices

        Ok(MeshIO {
            positions,
            uvs,
            normals,
            face_indices,
            face_has_uv,
            face_has_normal,
            line_indices,
            face_starting,
            end_of_indices,
        })
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
        face_starting: &mut Vec<usize>,
        end_of_indices: &mut Vec<(usize, usize, usize)>,
    ) -> Result<(), MeshIOError> {
        if line.starts_with('#') {
            return Ok(());
        }
        let vals: Vec<&str> = line.split(' ').collect();
        let indice = end_of_indices.last().unwrap();
        
        let (mut v_cnt, mut uv_cnt, mut n_cnt) = *indice;
        
        assert!(!vals.is_empty());
        match vals[0] {
            "v" => {
                // Don't currently support positions with 4 or more coordinates
                assert!(vals.len() == 4);
                let x: f64 = vals[1].parse().unwrap();
                let y: f64 = vals[2].parse().unwrap();
                let z: f64 = vals[3].parse().unwrap();
                positions.push(glm::vec3(x, y, z));
                v_cnt += 1;
                Ok(())
            }
            "vn" => {
                // Don't currently support positions with 4 or more coordinates
                assert!(vals.len() == 4);
                let x: f64 = vals[1].parse().unwrap();
                let y: f64 = vals[2].parse().unwrap();
                let z: f64 = vals[3].parse().unwrap();
                normals.push(glm::vec3(x, y, z));
                n_cnt += 1;
                Ok(())
            }
            "vt" => {
                // Don't currently support texture coordinates with 3 or more coordinates
                assert!(vals.len() == 3);
                let u: f64 = vals[1].parse().unwrap();
                let v: f64 = vals[2].parse().unwrap();
                uvs.push(glm::vec2(u, v));
                uv_cnt += 1;
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
                assert!(face_indices.len() >= 2);
                face_starting.push(face_indices.len());
                end_of_indices.push((v_cnt, uv_cnt, n_cnt));
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
}
