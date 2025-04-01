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
    pub fn bind(&mut self) {
        unsafe { gl::BindVertexArray(self.id) };
    }

    pub fn unbind(&mut self) {
        unsafe { gl::BindVertexArray(NULL_HANDLE) };
    }
    pub fn set_attribute(
        &mut self,
        location: GLuint,
        attribute: &VertexAttribute,
        offset: GLint,
        stride: GLsizei,
    ) {
        // Set the VertexAttribute pointer in this location

        let components = attribute.size;
        let data_type = attribute.data_type as GLenum;
        let normalized = if attribute.normalized {
            gl::TRUE
        } else {
            gl::FALSE
        };

        // Compute the attribute pointer
        let mut pointer = std::ptr::null_mut::<u8>(); // Actual base pointer is in VBO
        pointer = pointer.wrapping_add(offset as usize);

        if attribute.is_floating_point() || attribute.normalized {
            unsafe {
                gl::VertexAttribPointer(
                    location,
                    components,
                    data_type,
                    normalized,
                    stride,
                    pointer as *const c_void,
                );
            };
        } else {
            unsafe {
                gl::VertexAttribIPointer(
                    location,
                    components,
                    data_type,
                    stride,
                    pointer as *const c_void,
                )
            };
        }

        // Finally, we enable the VertexAttribute in this location
        unsafe { gl::EnableVertexAttribArray(location) };
    }
}
