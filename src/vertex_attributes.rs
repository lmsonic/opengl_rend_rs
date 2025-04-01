use std::os::raw::c_void;

use gl::types::{GLboolean, GLenum, GLint, GLsizei, GLuint};

use crate::{GLHandle, NULL_HANDLE};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DataType {
    Byte = gl::BYTE,
    UnsignedByte = gl::UNSIGNED_BYTE,
    Short = gl::SHORT,
    UnsignedShort = gl::UNSIGNED_SHORT,
    Int = gl::INT,
    UnsignedInt = gl::UNSIGNED_INT,
    Double = gl::DOUBLE,
    Float = gl::FLOAT,
    Fixed = gl::FIXED,
}

pub struct VertexAttribute {
    size: GLint,
    data_type: DataType,
    normalized: bool,
}

impl VertexAttribute {
    pub fn new(size: GLint, data_type: DataType, normalized: bool) -> Self {
        Self {
            size,
            data_type,
            normalized,
        }
    }

    pub fn is_floating_point(&self) -> bool {
        self.data_type == DataType::Float
            || self.data_type == DataType::Double
            || self.data_type == DataType::Fixed
    }
}

pub struct VertexArrayObject {
    id: GLHandle,
}

impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        unsafe { gl::DeleteVertexArrays(1, &self.id) };
    }
}
impl VertexArrayObject {
    pub fn new() -> Self {
        let mut id = NULL_HANDLE;
        unsafe { gl::GenVertexArrays(1, &mut id) };
        Self { id }
    }
    pub fn bind(&self) {
        unsafe { gl::BindVertexArray(self.id) };
    }

    pub fn unbind(&self) {
        unsafe { gl::BindVertexArray(NULL_HANDLE) };
    }
    pub fn set_attribute(
        &mut self,
        location: GLuint,
        attribute: &VertexAttribute,
        stride: GLint,
        offset: GLint,
    ) {
        self.bind();
        unsafe {
            gl::VertexAttribPointer(
                location,
                attribute.size,
                attribute.data_type as GLenum,
                if attribute.normalized {
                    gl::TRUE
                } else {
                    gl::FALSE
                },
                stride,
                offset as *const _,
            )
        }

        // Finally, we enable the VertexAttribute in this location
        unsafe { gl::EnableVertexAttribArray(location) };
    }
}
