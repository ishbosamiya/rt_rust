use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::glm;
use crate::util::str_to_cstr;

pub mod builtins;

pub struct Shader {
    program_id: gl::types::GLuint,
}

#[derive(Debug, Clone)]
pub enum ShaderError {
    Io,
    VertexCompile(String),
    FragmentCompile(String),
    ProgramLinker,
}

impl std::fmt::Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderError::Io => write!(f, "io error"),
            ShaderError::VertexCompile(error_log) => {
                write!(f, "vertex shader compile error with log: {}", error_log)
            }
            ShaderError::FragmentCompile(error_log) => {
                write!(f, "fragment shader compile error with log: {}", error_log)
            }
            ShaderError::ProgramLinker => write!(f, "program_linker error"),
        }
    }
}

impl std::error::Error for ShaderError {}

fn get_shader_error_log(shader: gl::types::GLuint) -> String {
    let mut max_length = 0;
    unsafe {
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut max_length);
    }

    let mut log: Vec<u8> = vec![0; max_length.try_into().unwrap()];

    unsafe {
        gl::GetShaderInfoLog(
            shader,
            max_length,
            &mut max_length,
            log.as_mut_ptr() as *mut gl::types::GLchar,
        );
    }

    String::from_utf8_lossy(&log[..max_length.try_into().unwrap()]).to_string()
}

impl Shader {
    pub fn new(
        vertex_shader_path: &Path,
        fragment_shader_path: &Path,
    ) -> Result<Shader, ShaderError> {
        let mut v_file = match File::open(vertex_shader_path) {
            Err(_) => return Err(ShaderError::Io),
            Ok(file) => file,
        };
        let mut f_file = match File::open(fragment_shader_path) {
            Err(_) => return Err(ShaderError::Io),
            Ok(file) => file,
        };

        let mut vertex_code = String::new();
        let mut fragment_code = String::new();
        if v_file.read_to_string(&mut vertex_code).is_err() {
            return Err(ShaderError::Io);
        }
        if f_file.read_to_string(&mut fragment_code).is_err() {
            return Err(ShaderError::Io);
        }

        Self::from_strings(&vertex_code, &fragment_code)
    }

    pub fn from_strings(vertex_code: &str, fragment_code: &str) -> Result<Shader, ShaderError> {
        let vertex_code = std::ffi::CString::new(vertex_code).unwrap();
        let vertex_shader: gl::types::GLuint;
        unsafe {
            vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex_shader, 1, &vertex_code.as_ptr(), std::ptr::null());
            gl::CompileShader(vertex_shader);
        }
        unsafe {
            let mut success: gl::types::GLint = -10;
            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE.into() {
                eprintln!("vertex didn't compile");

                let log = get_shader_error_log(vertex_shader);

                return Err(ShaderError::VertexCompile(log));
            }
        }
        let fragment_code = std::ffi::CString::new(fragment_code).unwrap();
        let fragment_shader: gl::types::GLuint;
        unsafe {
            fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(
                fragment_shader,
                1,
                &fragment_code.as_ptr(),
                std::ptr::null(),
            );
            gl::CompileShader(fragment_shader);
        }
        unsafe {
            let mut success: gl::types::GLint = -10;
            gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE.into() {
                eprintln!("fragment didn't compile");

                let log = get_shader_error_log(fragment_shader);

                return Err(ShaderError::FragmentCompile(log));
            }
        }
        let shader_program: gl::types::GLuint;
        unsafe {
            shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);
        }
        unsafe {
            let mut success: gl::types::GLint = -10;
            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE.into() {
                eprintln!("program not linked");
                return Err(ShaderError::ProgramLinker);
            }
        }

        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }

        Ok(Shader {
            program_id: shader_program,
        })
    }

    pub fn use_shader(&self) {
        unsafe {
            gl::UseProgram(self.program_id);
        }
    }

    pub fn set_bool(&self, name: &str, value: bool) {
        unsafe {
            if value {
                gl::Uniform1i(
                    gl::GetUniformLocation(self.program_id, str_to_cstr(name).as_ptr()),
                    1,
                );
            } else {
                gl::Uniform1i(
                    gl::GetUniformLocation(self.program_id, str_to_cstr(name).as_ptr()),
                    0,
                );
            }
        }
    }

    pub fn set_int(&self, name: &str, value: gl::types::GLint) {
        unsafe {
            gl::Uniform1i(
                gl::GetUniformLocation(self.program_id, str_to_cstr(name).as_ptr()),
                value,
            );
        }
    }

    pub fn set_float(&self, name: &str, value: gl::types::GLfloat) {
        unsafe {
            gl::Uniform1f(
                gl::GetUniformLocation(self.program_id, str_to_cstr(name).as_ptr()),
                value,
            );
        }
    }

    pub fn set_vec2(&self, name: &str, value: &glm::Vec2) {
        unsafe {
            gl::Uniform2f(
                gl::GetUniformLocation(self.program_id, str_to_cstr(name).as_ptr()),
                value[0],
                value[1],
            );
        }
    }

    pub fn set_vec3(&self, name: &str, value: &glm::Vec3) {
        unsafe {
            gl::Uniform3f(
                gl::GetUniformLocation(self.program_id, str_to_cstr(name).as_ptr()),
                value[0],
                value[1],
                value[2],
            );
        }
    }

    pub fn set_vec4(&self, name: &str, value: &glm::Vec4) {
        unsafe {
            gl::Uniform4f(
                gl::GetUniformLocation(self.program_id, str_to_cstr(name).as_ptr()),
                value[0],
                value[1],
                value[2],
                value[3],
            );
        }
    }

    pub fn set_mat2(&self, name: &str, value: &glm::Mat2) {
        unsafe {
            gl::UniformMatrix2fv(
                gl::GetUniformLocation(self.program_id, str_to_cstr(name).as_ptr()),
                1,
                gl::FALSE,
                value.as_ptr(),
            );
        }
    }

    pub fn set_mat3(&self, name: &str, value: &glm::Mat3) {
        unsafe {
            gl::UniformMatrix3fv(
                gl::GetUniformLocation(self.program_id, str_to_cstr(name).as_ptr()),
                1,
                gl::FALSE,
                value.as_ptr(),
            );
        }
    }

    pub fn set_mat4(&self, name: &str, value: &glm::Mat4) {
        unsafe {
            gl::UniformMatrix4fv(
                gl::GetUniformLocation(self.program_id, str_to_cstr(name).as_ptr()),
                1,
                gl::FALSE,
                value.as_ptr(),
            );
        }
    }

    pub fn get_id(&self) -> gl::types::GLuint {
        self.program_id
    }

    pub fn get_attributes(&self) -> Vec<String> {
        let mut attributes = Vec::new();
        let mut count: gl::types::GLint = -1;
        const MAX_LENGTH: usize = 100;
        let mut name: [i8; MAX_LENGTH] = [-1; MAX_LENGTH];
        let mut length: gl::types::GLsizei = -1;
        let mut size: gl::types::GLint = -1;
        let mut var_type: gl::types::GLenum = gl::NONE;
        unsafe {
            gl::GetProgramiv(self.get_id(), gl::ACTIVE_ATTRIBUTES, &mut count);
        }

        for i in 0..count {
            unsafe {
                gl::GetActiveAttrib(
                    self.get_id(),
                    i.try_into().unwrap(),
                    MAX_LENGTH.try_into().unwrap(),
                    &mut length,
                    &mut size,
                    &mut var_type,
                    name.as_mut_ptr(),
                );
            }
            let name_string: std::ffi::CString;
            unsafe {
                name_string = std::ffi::CString::from(std::ffi::CStr::from_ptr(name.as_ptr()));
            }
            attributes.push(name_string.into_string().unwrap());
        }

        attributes
    }

    pub fn get_uniforms(&self) -> Vec<String> {
        let mut uniforms = Vec::new();
        let mut count: gl::types::GLint = -1;
        const MAX_LENGTH: usize = 100;
        let mut name: [i8; MAX_LENGTH] = [-1; MAX_LENGTH];
        let mut length: gl::types::GLsizei = -1;
        let mut size: gl::types::GLint = -1;
        let mut var_type: gl::types::GLenum = gl::NONE;
        unsafe {
            gl::GetProgramiv(self.get_id(), gl::ACTIVE_UNIFORMS, &mut count);
        }

        for i in 0..count {
            unsafe {
                gl::GetActiveUniform(
                    self.get_id(),
                    i.try_into().unwrap(),
                    MAX_LENGTH.try_into().unwrap(),
                    &mut length,
                    &mut size,
                    &mut var_type,
                    name.as_mut_ptr(),
                );
            }
            let name_string: std::ffi::CString;
            unsafe {
                name_string = std::ffi::CString::from(std::ffi::CStr::from_ptr(name.as_ptr()));
            }
            uniforms.push(name_string.into_string().unwrap());
        }

        uniforms
    }
}
