use std::os::raw::c_void;

use gl::types::{GLboolean, GLenum, GLint, GLsizei, GLuint};

#[derive(Clone, Copy)]
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
    index: GLuint,
    size: GLint,
    type_: DataType,
    normalized: GLboolean,
    stride: GLsizei,
    pointer: *const c_void,
}

impl VertexAttribute {
    pub fn new(
        index: GLuint,
        size: GLint,
        type_: DataType,
        normalized: GLboolean,
        stride: GLsizei,
        pointer: *const c_void,
    ) -> Self {
        Self {
            index,
            size,
            type_,
            normalized,
            stride,
            pointer,
        }
    }

    pub fn enable(&self) {
        unsafe { gl::EnableVertexAttribArray(self.index) }
    }
    pub fn create(&self) {
        unsafe {
            gl::VertexAttribPointer(
                self.index,
                self.size,
                self.type_ as GLenum,
                self.normalized,
                self.stride,
                self.pointer,
            )
        }
    }

    pub fn disable(&self) {
        unsafe { gl::EnableVertexAttribArray(self.index) }
    }
}
