use std::marker::PhantomData;

use gl::types::{GLenum, GLintptr, GLsizeiptr, GLuint};

use crate::{GLHandle, NULL_HANDLE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Target {
    ArrayBuffer = gl::ARRAY_BUFFER,
    IndexBuffer = gl::ELEMENT_ARRAY_BUFFER,

    AtomicCounterBuffer = gl::ATOMIC_COUNTER_BUFFER,
    CopyReadBuffer = gl::COPY_READ_BUFFER,
    CopyWriteBuffer = gl::COPY_WRITE_BUFFER,
    DispatchIndirectBuffer = gl::DISPATCH_INDIRECT_BUFFER,
    DrawIndirectBuffer = gl::DRAW_INDIRECT_BUFFER,
    PixelPackBuffer = gl::PIXEL_PACK_BUFFER,
    PixelUnpackBuffer = gl::PIXEL_UNPACK_BUFFER,
    QueryBuffer = gl::QUERY_BUFFER,
    ShaderStorageBuffer = gl::SHADER_STORAGE_BUFFER,
    TextureBuffer = gl::TEXTURE_BUFFER,
    TransformFeedbackBuffer = gl::TRANSFORM_FEEDBACK_BUFFER,
    UniformBuffer = gl::UNIFORM_BUFFER,
}

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum Usage {
    StaticDraw = gl::STATIC_DRAW,
    DynamicDraw = gl::DYNAMIC_DRAW,

    StreamDraw = gl::STREAM_DRAW,
    StreamRead = gl::STREAM_READ,
    StreamCopy = gl::STREAM_COPY,
    StaticRead = gl::STATIC_READ,
    StaticCopy = gl::STATIC_COPY,
    DynamicRead = gl::DYNAMIC_READ,
    DynamicCopy = gl::DYNAMIC_COPY,
}

pub struct Buffer<T: Default> {
    id: GLHandle,
    target: Target,
    phantom: PhantomData<T>,
}

impl<T: Default> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.id) }
    }
}

impl<T: Default> Buffer<T> {
    pub fn new(target: Target) -> Self {
        let mut id = NULL_HANDLE;
        unsafe { gl::GenBuffers(1, &mut id) };
        Self {
            id,
            target,
            phantom: PhantomData,
        }
    }
    pub fn reserve_data(&mut self, size: isize, usage: Usage) {
        let size_bytes = size * std::mem::size_of::<T>() as isize;
        unsafe {
            gl::BufferData(
                self.target as GLenum,
                size_bytes as isize,
                std::ptr::null(),
                usage as GLenum,
            )
        };
    }
    pub fn reserve_data_bytes(&mut self, size: GLsizeiptr, usage: Usage) {
        unsafe {
            gl::BufferData(
                self.target as GLenum,
                size,
                std::ptr::null(),
                usage as GLenum,
            )
        };
    }

    pub fn buffer_data(&mut self, data: &[T], usage: Usage) {
        unsafe {
            gl::BufferData(
                self.target as GLenum,
                std::mem::size_of_val(data) as isize,
                data.as_ptr() as *const _,
                usage as GLenum,
            )
        };
    }
    pub fn get_data(&mut self, offset: isize, size: usize) -> Vec<T> {
        let mut data: Vec<T> = vec![];
        for _ in 0..size {
            data.push(T::default());
        }

        let size_bytes = size * std::mem::size_of::<T>();
        let offset_bytes = offset * std::mem::size_of::<T>() as isize;
        dbg!(size_bytes);
        unsafe {
            gl::GetBufferSubData(
                self.target as GLenum,
                offset_bytes as GLintptr,
                size_bytes as isize,
                data.as_mut_ptr() as _,
            )
        };
        data
    }
    pub fn update_data(&mut self, data: &[T], offset: isize) {
        let offset_bytes = offset * std::mem::size_of::<T>() as isize;

        unsafe {
            gl::BufferSubData(
                self.target as GLenum,
                offset_bytes as GLintptr,
                std::mem::size_of_val(data) as isize,
                data.as_ptr() as *const _,
            )
        };
    }

    pub fn update_data_bytes(&mut self, data: &[u8], size: GLsizeiptr, offset: GLintptr) {
        unsafe {
            gl::BufferSubData(
                self.target as GLenum,
                offset,
                size,
                data.as_ptr() as *const _,
            )
        };
    }

    pub fn bind_range(&mut self, binding_index: GLuint, offset: isize, size: usize) {
        assert!(
            self.target == Target::AtomicCounterBuffer
                || self.target == Target::TransformFeedbackBuffer
                || self.target == Target::UniformBuffer
                || self.target == Target::ShaderStorageBuffer
        );
        let size_bytes = size * std::mem::size_of::<T>();
        let offset_bytes = offset * std::mem::size_of::<T>() as isize;

        unsafe {
            gl::BindBufferRange(
                self.target as GLenum,
                binding_index,
                self.id,
                offset_bytes as GLsizeiptr,
                size_bytes as GLsizeiptr,
            )
        };
    }
    pub fn bind_range_bytes(&mut self, binding_index: GLuint, offset: GLintptr, size: GLsizeiptr) {
        assert!(
            self.target == Target::AtomicCounterBuffer
                || self.target == Target::TransformFeedbackBuffer
                || self.target == Target::UniformBuffer
                || self.target == Target::ShaderStorageBuffer
        );
        unsafe { gl::BindBufferRange(self.target as GLenum, binding_index, self.id, offset, size) };
    }

    pub fn bind(&mut self) {
        unsafe { gl::BindBuffer(self.target as GLenum, self.id) };
    }
    pub fn unbind(&mut self) {
        unsafe { gl::BindBuffer(self.target as GLenum, NULL_HANDLE) };
    }
}
