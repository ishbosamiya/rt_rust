// Based on Blender's GPU Immediate work-alike system

use std::convert::TryInto;

use crate::rasterize::shader::Shader;
use crate::util::str_to_cstr;

const GPU_VERT_ATTR_MAX_LEN: usize = 16;
const IMM_DEFAULT_BUFFER_SIZE: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub enum GPUVertCompType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    F32,
    I10,
    None,
}

impl GPUVertCompType {
    fn to_gl(&self) -> gl::types::GLenum {
        match self {
            GPUVertCompType::I8 => gl::BYTE,
            GPUVertCompType::U8 => gl::UNSIGNED_BYTE,
            GPUVertCompType::I16 => gl::SHORT,
            GPUVertCompType::U16 => gl::UNSIGNED_SHORT,
            GPUVertCompType::I32 => gl::INT,
            GPUVertCompType::U32 => gl::UNSIGNED_INT,
            GPUVertCompType::F32 => gl::FLOAT,
            GPUVertCompType::I10 => gl::INT_2_10_10_10_REV,
            GPUVertCompType::None => panic!("GPUVertCompType shouldn't be None"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GPUVertFetchMode {
    Float,
    Int,
    IntToFloatUnit, // eg: 127 (ubyte) -> 0.5
    IntToFloat,     // eg: 127 (ubyte) -> 127.0
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GPUPrimType {
    Points,
    Lines,
    Tris,
    LineStrip,
    LineLoop,
    TriStrip,
    TriFan,

    LinesAdj,
    TrisAdj,
    LineStripAdj,

    None,
}

impl GPUPrimType {
    fn to_gl(&self) -> gl::types::GLenum {
        match self {
            GPUPrimType::Points => gl::POINTS,
            GPUPrimType::Lines => gl::LINES,
            GPUPrimType::Tris => gl::TRIANGLES,
            GPUPrimType::LineStrip => gl::LINE_STRIP,
            GPUPrimType::LineLoop => gl::LINE_LOOP,
            GPUPrimType::TriStrip => gl::TRIANGLE_STRIP,
            GPUPrimType::TriFan => gl::TRIANGLE_FAN,

            GPUPrimType::LinesAdj => gl::LINES_ADJACENCY,
            GPUPrimType::TrisAdj => gl::TRIANGLES_ADJACENCY,
            GPUPrimType::LineStripAdj => gl::LINE_STRIP_ADJACENCY,

            GPUPrimType::None => panic!("GPUPrimType shouldn't be None when to_gl() is called"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GPUAttrBinding {
    loc_bits: u64,     // store 4 bits for each the 16 attributes
    enabled_bits: u16, // store 1 bit for each attribute
}

impl GPUAttrBinding {
    fn new() -> Self {
        GPUAttrBinding {
            loc_bits: 0,
            enabled_bits: 0,
        }
    }

    fn clear(&mut self) {
        self.loc_bits = 0;
        self.enabled_bits = 0;
    }

    fn write_attr_location(&mut self, attr_index: usize, location: usize) {
        let shift = 4 * attr_index;
        let mask = 0xF << shift;

        self.loc_bits = (self.loc_bits & !mask) | (location << shift) as u64;
        self.enabled_bits |= 1 << attr_index;
    }

    fn read_attr_location(&self, attr_index: usize) -> usize {
        ((self.loc_bits >> (4 * attr_index)) & 0xF)
            .try_into()
            .unwrap()
    }
}

#[derive(Debug, Clone)]
struct GPUVertAttr {
    fetch_mode: GPUVertFetchMode,
    comp_type: GPUVertCompType,
    comp_len: u8,
    sz: u8,
    offset: u8,
    gl_comp_type: gl::types::GLenum,
    name: String,
}

impl GPUVertAttr {
    fn new() -> Self {
        GPUVertAttr {
            fetch_mode: GPUVertFetchMode::None,
            comp_type: GPUVertCompType::None,
            comp_len: 0,
            sz: 0,
            offset: 0,
            gl_comp_type: gl::NONE,
            name: String::new(),
        }
    }

    fn comp_sz(&self, r#type: &GPUVertCompType) -> u8 {
        match r#type {
            GPUVertCompType::I8 => 1,
            GPUVertCompType::U8 => 1,
            GPUVertCompType::I16 => 2,
            GPUVertCompType::U16 => 2,
            GPUVertCompType::I32 => 4,
            GPUVertCompType::U32 => 4,
            GPUVertCompType::F32 => 4,
            GPUVertCompType::I10 => 4,
            GPUVertCompType::None => panic!("GPUVertCompType shouldn't be None"),
        }
    }

    fn attr_sz(&self) -> u8 {
        if let GPUVertCompType::I10 = self.comp_type {
            4 // always packed as 10_10_10_2
        } else {
            self.comp_len * self.comp_sz(&self.comp_type)
        }
    }

    fn attr_align(&self) -> usize {
        if let GPUVertCompType::I10 = self.comp_type {
            4 // always packed as 10_10_10_2
        } else {
            let c = self.comp_sz(&self.comp_type);
            if self.comp_len == 3 && c < 2 {
                (4 * c).into() // AMD HW can't fetch these well, so pad it out (other vendors too?)
            } else {
                c.into() // most fetches are ok if components are naturally aligned
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GPUVertFormat {
    stride: u16,  // stride in bytes 1 to 1024
    packed: bool, // has the format been packed

    attrs: Vec<GPUVertAttr>,
}

fn padding(offset: usize, alignment: usize) -> usize {
    let m = offset % alignment;
    if m == 0 {
        0
    } else {
        alignment - m
    }
}

impl GPUVertFormat {
    fn new() -> Self {
        GPUVertFormat {
            stride: 0,
            packed: false,

            attrs: Vec::new(),
        }
    }

    fn pack(&mut self) {
        // For now, attributes are packed in the order they were added, making sure each attribute is naturally aligned (add padding where necessary). Later we can implement more efficient packing w/ reordering (keep attribute ID order, adjust their offsets to reorder in buffer).
        let mut offset: usize;
        {
            let mut a0 = &mut self.attrs[0];
            a0.offset = 0;
            offset = a0.sz.into();
        }

        for a in self.attrs.iter_mut().skip(1) {
            let mid_padding = padding(offset, a.attr_align());
            offset += mid_padding;
            a.offset = offset.try_into().unwrap();
            let temp_size: usize = a.sz.into();
            offset += temp_size;
        }

        let end_padding = padding(offset, self.attrs[0].attr_align());

        self.stride = (offset + end_padding).try_into().unwrap();
        self.packed = true;
    }

    fn vertex_buffer_size(&self, vertex_len: usize) -> usize {
        self.stride as usize * vertex_len
    }

    pub fn add_attribute(
        &mut self,
        name: String,
        comp_type: GPUVertCompType,
        comp_len: usize,
        fetch_mode: GPUVertFetchMode,
    ) -> usize {
        // TODO(ish): add asserts
        let mut attr = GPUVertAttr::new();

        attr.name = name;
        attr.comp_type = comp_type;
        attr.gl_comp_type = attr.comp_type.to_gl();
        attr.comp_len = comp_len.try_into().unwrap();
        if let GPUVertCompType::I10 = attr.comp_type {
            attr.comp_len = 4;
        }
        attr.sz = attr.attr_sz();
        attr.offset = 0; // will be calculated during pack()
        attr.fetch_mode = fetch_mode;

        self.attrs.push(attr);

        self.attrs.len() - 1
        // TODO(ish): this is returning a value within self.attrs which doesn't have to correspond with the value in the vertex shader. Need to figure out what is happening
    }

    pub fn clear(&mut self) {
        self.attrs.clear();
        self.packed = false;
        self.attrs.iter_mut().for_each(|attr| attr.name.clear());
    }
}

#[derive(Debug, Clone)]
pub struct GPUImmediate {
    buffer_data: *mut gl::types::GLubyte,
    buffer_offset: usize,
    buffer_bytes_mapped: usize,
    vertex_len: usize,
    strict_vertex_len: bool,
    prim_type: GPUPrimType,
    buffer_size: usize,

    vertex_format: GPUVertFormat,

    vertex_idx: usize,
    vertex_data: *mut gl::types::GLubyte,
    unassigned_attr_bits: u16, // which attributes of the current vertex have not been given values
    vbo_id: gl::types::GLuint,
    vao_id: gl::types::GLuint,

    attr_binding: GPUAttrBinding,
    prev_enabled_attr_bits: u16, // affects only this vao
}

fn gpu_buf_alloc(id: &mut gl::types::GLuint) {
    unsafe {
        gl::GenBuffers(1, id);
    }
}

fn gpu_buf_free(id: &gl::types::GLuint) {
    unsafe {
        gl::DeleteBuffers(1, id);
    }
}

fn gpu_vao_alloc(id: &mut gl::types::GLuint) {
    unsafe {
        gl::GenVertexArrays(1, id);
    }
}

fn gpu_vao_free(id: &gl::types::GLuint) {
    unsafe {
        gl::DeleteVertexArrays(1, id);
    }
}

impl Default for GPUImmediate {
    fn default() -> Self {
        Self::new()
    }
}

impl GPUImmediate {
    pub fn new() -> Self {
        let mut imm = GPUImmediate {
            buffer_data: std::ptr::null_mut(),
            buffer_offset: 0,
            buffer_bytes_mapped: 0,
            vertex_len: 0,
            strict_vertex_len: false,
            prim_type: GPUPrimType::None,
            buffer_size: IMM_DEFAULT_BUFFER_SIZE,

            vertex_format: GPUVertFormat::new(),

            vertex_idx: 0,
            vertex_data: std::ptr::null_mut(),
            unassigned_attr_bits: 0,
            vbo_id: 0,
            vao_id: 0,

            attr_binding: GPUAttrBinding::new(),
            prev_enabled_attr_bits: 0,
        };

        imm.init();

        imm
    }

    fn init(&mut self) {
        gpu_buf_alloc(&mut self.vbo_id);
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_id);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                self.buffer_size.try_into().unwrap(),
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );
        }

        self.prim_type = GPUPrimType::None;
        self.strict_vertex_len = true;

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        // activate vao
        gpu_vao_alloc(&mut self.vao_id);
    }

    pub fn begin(&mut self, prim_type: GPUPrimType, vertex_len: usize, shader: &Shader) {
        assert_ne!(vertex_len, 0);

        if !self.vertex_format.packed {
            self.vertex_format.pack();
        }

        // TODO(ish): need to get attribute locations and enable the correct attributes
        self.attr_binding.clear();

        for (attr_index, a) in self.vertex_format.attrs.iter().enumerate() {
            let location;
            unsafe {
                location = gl::GetAttribLocation(shader.get_id(), str_to_cstr(&a.name).as_ptr());
            }
            self.attr_binding
                .write_attr_location(attr_index, location.try_into().unwrap());
        }

        self.prim_type = prim_type;
        self.vertex_len = vertex_len;
        self.vertex_idx = 0;
        self.unassigned_attr_bits = self.attr_binding.enabled_bits;

        let bytes_needed = self.vertex_format.vertex_buffer_size(self.vertex_len);

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_id);
        }

        let available_bytes = self.buffer_size - self.buffer_offset;

        let recreate_buffer;
        if bytes_needed > self.buffer_size {
            // expand buffer
            self.buffer_size = bytes_needed;
            recreate_buffer = true;
        } else if bytes_needed < IMM_DEFAULT_BUFFER_SIZE
            && self.buffer_size > IMM_DEFAULT_BUFFER_SIZE
        {
            // shrink buffer
            self.buffer_size = IMM_DEFAULT_BUFFER_SIZE;
            recreate_buffer = true;
        } else {
            // no change to size of buffer
            recreate_buffer = false;
        }

        let pre_padding = padding(self.buffer_offset, self.vertex_format.stride.into());

        if !recreate_buffer && ((bytes_needed + pre_padding) <= available_bytes) {
            self.buffer_offset += pre_padding;
        } else {
            // create a new buffer
            unsafe {
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    self.buffer_size.try_into().unwrap(),
                    std::ptr::null(),
                    gl::DYNAMIC_DRAW,
                );
            }
            self.buffer_offset = 0;
        }

        unsafe {
            if self.strict_vertex_len {
                self.buffer_data = gl::MapBufferRange(
                    gl::ARRAY_BUFFER,
                    self.buffer_offset.try_into().unwrap(),
                    bytes_needed.try_into().unwrap(),
                    gl::MAP_WRITE_BIT | gl::MAP_UNSYNCHRONIZED_BIT,
                ) as *mut gl::types::GLubyte;
            } else {
                self.buffer_data = gl::MapBufferRange(
                    gl::ARRAY_BUFFER,
                    self.buffer_offset.try_into().unwrap(),
                    bytes_needed.try_into().unwrap(),
                    gl::MAP_WRITE_BIT | gl::MAP_UNSYNCHRONIZED_BIT | gl::MAP_FLUSH_EXPLICIT_BIT,
                ) as *mut gl::types::GLubyte;
            }
        }

        if self.buffer_data.is_null() {
            loop {
                let error;
                unsafe {
                    error = gl::GetError();
                }
                if error == gl::NO_ERROR {
                    break;
                } else if error == gl::INVALID_ENUM {
                    eprintln!("opengl error: gl::INVALID_ENUM");
                } else if error == gl::INVALID_VALUE {
                    eprintln!("opengl error: gl::INVALID_VALUE");
                } else if error == gl::INVALID_OPERATION {
                    eprintln!("opengl error: gl::INVALID_OPERATION");
                } else if error == gl::INVALID_FRAMEBUFFER_OPERATION {
                    eprintln!("opengl error: gl::INVALID_FRAMEBUFFER_OPERATION");
                } else if error == gl::OUT_OF_MEMORY {
                    eprintln!("opengl error: gl::OUT_OF_MEMORY");
                } else if error == gl::STACK_UNDERFLOW {
                    eprintln!("opengl error: gl::STACK_UNDERFLOW");
                } else if error == gl::STACK_OVERFLOW {
                    eprintln!("opengl error: gl::STACK_OVERFLOW");
                } else {
                    panic!("should have been one of the above opengl errors");
                }
            }
        }

        assert_ne!(self.buffer_data, std::ptr::null_mut());

        self.buffer_bytes_mapped = bytes_needed;
        self.vertex_data = self.buffer_data;
    }

    pub fn begin_at_most(&mut self, prim_type: GPUPrimType, vertex_len: usize, shader: &Shader) {
        self.strict_vertex_len = false;
        self.begin(prim_type, vertex_len, shader);
    }

    pub fn end(&mut self) {
        assert_ne!(self.prim_type, GPUPrimType::None); // to ensure end isn't called before begin

        let buffer_bytes_used;
        if self.strict_vertex_len {
            assert_eq!(self.vertex_idx, self.vertex_len, "number of verts is not equal to promised vertex_len; self.vertex_idx: {}, self.vertex_len: {}", self.vertex_idx, self.vertex_len);
            buffer_bytes_used = self.buffer_bytes_mapped;
        } else {
            assert!(self.vertex_idx <= self.vertex_len, "number of verts exceeded promised vertex_len; self.vertex_idx: {}, self.vertex_len: {}", self.vertex_idx, self.vertex_len);

            if self.vertex_idx == self.vertex_len {
                buffer_bytes_used = self.buffer_bytes_mapped;
            } else {
                self.vertex_len = self.vertex_idx;
                buffer_bytes_used = self.vertex_format.vertex_buffer_size(self.vertex_len);
            }

            unsafe {
                gl::FlushMappedBufferRange(
                    gl::ARRAY_BUFFER,
                    0,
                    buffer_bytes_used.try_into().unwrap(),
                );
            }
        }

        unsafe {
            gl::UnmapBuffer(gl::ARRAY_BUFFER);
        }

        if self.vertex_len > 0 {
            self.draw_setup();

            #[cfg(target_os = "macos")]
            unsafe {
                gl::Disable(gl::PRIMITIVE_RESTART);
            }

            unsafe {
                gl::DrawArrays(
                    self.prim_type.to_gl(),
                    0,
                    self.vertex_len.try_into().unwrap(),
                );
            }

            #[cfg(target_os = "macos")]
            unsafe {
                gl::Enable(gl::PRIMITIVE_RESTART);
            }

            self.buffer_offset += buffer_bytes_used;
        }

        // setup for next begin
        self.prim_type = GPUPrimType::None;
        self.strict_vertex_len = true;
    }

    fn draw_setup(&mut self) {
        unsafe {
            gl::BindVertexArray(self.vao_id);
        }

        if self.attr_binding.enabled_bits != self.prev_enabled_attr_bits {
            for loc in 0..GPU_VERT_ATTR_MAX_LEN {
                let is_enabled: bool = (self.attr_binding.enabled_bits & (1 << loc)) != 0;
                let was_enabled: bool = (self.prev_enabled_attr_bits & (1 << loc)) != 0;

                if is_enabled && !was_enabled {
                    unsafe {
                        gl::EnableVertexAttribArray(loc.try_into().unwrap());
                    }
                } else if was_enabled && !is_enabled {
                    unsafe {
                        gl::DisableVertexAttribArray(loc.try_into().unwrap());
                    }
                }
            }

            self.prev_enabled_attr_bits = self.attr_binding.enabled_bits;
        }

        let stride = self.vertex_format.stride;

        for (attr_index, a) in self.vertex_format.attrs.iter().enumerate() {
            let offset = self.buffer_offset + a.offset as usize;
            // let pointer: *mut gl::types::GLubyte = offset as *mut gl::types::GLubyte;
            let pointer: *mut gl::types::GLubyte;
            unsafe {
                let null: *mut gl::types::GLubyte = std::ptr::null_mut();
                pointer = null.offset(offset.try_into().unwrap());
            }

            let loc = self.attr_binding.read_attr_location(attr_index);

            match a.fetch_mode {
                GPUVertFetchMode::Float | GPUVertFetchMode::IntToFloat => unsafe {
                    gl::VertexAttribPointer(
                        loc.try_into().unwrap(),
                        a.comp_len.into(),
                        a.gl_comp_type,
                        gl::FALSE,
                        stride.into(),
                        pointer as *const gl::types::GLvoid,
                    );
                },
                GPUVertFetchMode::IntToFloatUnit => unsafe {
                    gl::VertexAttribPointer(
                        loc.try_into().unwrap(),
                        a.comp_len.into(),
                        a.gl_comp_type,
                        gl::TRUE,
                        stride.into(),
                        pointer as *const gl::types::GLvoid,
                    );
                },
                GPUVertFetchMode::Int => unsafe {
                    gl::VertexAttribIPointer(
                        loc.try_into().unwrap(),
                        a.comp_len.into(),
                        a.gl_comp_type,
                        stride.into(),
                        pointer as *const gl::types::GLvoid,
                    );
                },
                GPUVertFetchMode::None => panic!("GPUVertFetchMode shouldn't be None"),
            }
        }
    }

    pub fn get_cleared_vertex_format(&mut self) -> &mut GPUVertFormat {
        self.vertex_format.clear();
        &mut self.vertex_format
    }

    fn set_attr_value_bit(&mut self, attr_id: usize) {
        let mask = 1 << attr_id;
        self.unassigned_attr_bits &= !mask;
    }

    pub fn attr_1f(&mut self, attr_id: usize, x: f32) {
        // TODO(ish): add asserts
        self.set_attr_value_bit(attr_id);
        let attr = &self.vertex_format.attrs[attr_id];
        unsafe {
            let data = self.vertex_data.offset(attr.offset.into()) as *mut f32;
            *data = x;
        }
    }

    pub fn attr_2f(&mut self, attr_id: usize, x: f32, y: f32) {
        // TODO(ish): add asserts
        self.set_attr_value_bit(attr_id);
        let attr = &self.vertex_format.attrs[attr_id];
        unsafe {
            let data = self.vertex_data.offset(attr.offset.into()) as *mut f32;
            *data = x;
            *data.offset(1) = y;
        }
    }

    pub fn attr_3f(&mut self, attr_id: usize, x: f32, y: f32, z: f32) {
        // TODO(ish): add asserts
        self.set_attr_value_bit(attr_id);
        let attr = &self.vertex_format.attrs[attr_id];
        unsafe {
            let data = self.vertex_data.offset(attr.offset.into()) as *mut f32;
            *data = x;
            *data.offset(1) = y;
            *data.offset(2) = z;
        }
    }

    pub fn attr_4f(&mut self, attr_id: usize, x: f32, y: f32, z: f32, w: f32) {
        // TODO(ish): add asserts
        self.set_attr_value_bit(attr_id);
        let attr = &self.vertex_format.attrs[attr_id];
        unsafe {
            let data = self.vertex_data.offset(attr.offset.into()) as *mut f32;
            *data = x;
            *data.offset(1) = y;
            *data.offset(2) = z;
            *data.offset(3) = w;
        }
    }

    fn end_vertex(&mut self) {
        // TODO(ish): add asserts

        // if all attributes haven't been assigned, take from previous vertex
        if self.unassigned_attr_bits != 0 {
            assert!(self.vertex_idx > 0);

            for (attr_index, a) in self.vertex_format.attrs.iter().enumerate() {
                if (self.unassigned_attr_bits >> attr_index) & 1 != 0 {
                    unsafe {
                        let data = self.vertex_data.offset(a.offset.into());
                        let offset: isize = self.vertex_format.stride.try_into().unwrap();
                        std::ptr::copy_nonoverlapping(data.offset(-offset), data, a.sz.into());
                    }
                }
            }
        }

        self.vertex_idx += 1;
        unsafe {
            self.vertex_data = self
                .vertex_data
                .offset(self.vertex_format.stride.try_into().unwrap());
        }
        self.unassigned_attr_bits = self.attr_binding.enabled_bits;
    }

    pub fn vertex_2f(&mut self, attr_id: usize, x: f32, y: f32) {
        self.attr_2f(attr_id, x, y);
        self.end_vertex();
    }

    pub fn vertex_3f(&mut self, attr_id: usize, x: f32, y: f32, z: f32) {
        self.attr_3f(attr_id, x, y, z);
        self.end_vertex();
    }

    pub fn vertex_4f(&mut self, attr_id: usize, x: f32, y: f32, z: f32, w: f32) {
        self.attr_4f(attr_id, x, y, z, w);
        self.end_vertex();
    }
}

impl Drop for GPUImmediate {
    fn drop(&mut self) {
        gpu_buf_free(&self.vbo_id);
        // deactivate vao
        gpu_vao_free(&self.vao_id);
    }
}
