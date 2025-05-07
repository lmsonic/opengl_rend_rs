use gl::types::{GLenum, GLint, GLsizei, GLuint};

use crate::{opengl::IndexSize, GLHandle, NULL_HANDLE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl From<IndexSize> for DataType {
    fn from(value: IndexSize) -> Self {
        match value {
            IndexSize::UnsignedByte => Self::UnsignedByte,
            IndexSize::UnsignedShort => Self::UnsignedShort,
            IndexSize::UnsignedInt => Self::UnsignedInt,
        }
    }
}

impl DataType {
    #[must_use]
    pub const fn size(&self) -> usize {
        match self {
            Self::Byte | Self::UnsignedByte => 1,
            Self::Short | Self::UnsignedShort | Self::Fixed => 2,
            Self::Int | Self::UnsignedInt | Self::Float => 4,
            Self::Double => 8,
        }
    }
    #[must_use]
    pub fn is_floating_point(self) -> bool {
        self == Self::Float || self == Self::Double || self == Self::Fixed
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VertexAttribute {
    pub components: GLint,
    pub data_type: DataType,
    pub normalized: bool,
}

impl VertexAttribute {
    #[must_use]
    pub const fn new(components: GLint, data_type: DataType, normalized: bool) -> Self {
        Self {
            components,
            data_type,
            normalized,
        }
    }

    #[must_use]
    pub const fn size(&self) -> usize {
        self.data_type.size() * self.components as usize
    }

    #[must_use]
    pub fn is_floating_point(&self) -> bool {
        self.data_type.is_floating_point()
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
    #[must_use]
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
    pub fn unbind_all() {
        unsafe { gl::BindVertexArray(NULL_HANDLE) };
    }

    pub fn set_attribute(
        &mut self,
        location: GLuint,
        attribute: &VertexAttribute,
        stride: GLsizei,
        offset: GLint,
    ) {
        // Set the VertexAttribute pointer in this location

        let components = attribute.components;
        let data_type = attribute.data_type as GLenum;
        let normalized = if attribute.normalized {
            gl::TRUE
        } else {
            gl::FALSE
        };

        // Compute the attribute pointer
        let mut pointer = std::ptr::null_mut::<u8>(); // Actual base pointer is in VBO
        pointer = pointer.wrapping_add(offset as usize);
        // let pointer = offset;

        if attribute.is_floating_point() || attribute.normalized {
            unsafe {
                gl::VertexAttribPointer(
                    location,
                    components,
                    data_type,
                    normalized,
                    stride,
                    pointer as *const _,
                );
            };
        } else {
            // integral
            unsafe {
                gl::VertexAttribIPointer(
                    location,
                    components,
                    data_type,
                    stride,
                    pointer as *const _,
                );
            };
        }

        // Finally, we enable the VertexAttribute in this location
        unsafe { gl::EnableVertexAttribArray(location) };
    }
}

impl Default for VertexArrayObject {
    fn default() -> Self {
        Self::new()
    }
}
