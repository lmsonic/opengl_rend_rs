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
    // index: GLuint,
    size: GLint,
    data_type: DataType,
    normalized: bool,
    // stride: GLsizei,
    // pointer: *const c_void,
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
        index: GLuint,
        attribute: &VertexAttribute,
        stride: GLint,
        offset: GLsizei,
    ) {
        let size = attribute.size;
        let data_type = attribute.data_type;
        let normalized = if attribute.normalized {
            gl::TRUE
        } else {
            gl::FALSE
        };

        // Compute the attribute pointer
        let pointer = std::ptr::null_mut::<u8>(); // Actual base pointer is in VBO
        let pointer = pointer.wrapping_add(offset as usize) as *const c_void; // Set the VertexAttribute pointer in this location

        if attribute.normalized || attribute.is_floating_point() {
            unsafe {
                gl::VertexAttribPointer(
                    index,
                    size,
                    data_type as GLenum,
                    normalized,
                    stride,
                    pointer,
                )
            }
        } else {
            unsafe { gl::VertexAttribIPointer(index, size, data_type as GLenum, stride, pointer) }
        }
        // Finally, we enable the VertexAttribute in this location
        unsafe { gl::EnableVertexAttribArray(index) };
    }
}
