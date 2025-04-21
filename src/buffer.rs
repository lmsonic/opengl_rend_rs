use std::marker::PhantomData;

use gl::types::{GLenum, GLintptr, GLsizeiptr};

use crate::{GLHandle, NULL_HANDLE};

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum BufferType {
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

pub struct Buffer<T> {
    id: GLHandle,
    target: BufferType,
    phantom: PhantomData<T>,
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.id) }
    }
}

impl<T> Buffer<T> {
    pub fn new(target: BufferType) -> Self {
        let mut id = NULL_HANDLE;
        unsafe { gl::GenBuffers(1, &mut id) };
        Self {
            id,
            target,
            phantom: PhantomData,
        }
    }
    pub fn reserve_data(&mut self, size: GLsizeiptr, usage: Usage) {
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
    pub fn update_data(&mut self, data: &[T], offset: GLintptr) {
        dbg!(std::mem::size_of_val(data));
        dbg!(std::mem::size_of::<T>());

        unsafe {
            gl::BufferSubData(
                self.target as GLenum,
                offset,
                std::mem::size_of_val(data) as isize,
                data.as_ptr() as *const _,
            )
        };
    }

    pub fn update_data_custom_size(&mut self, data: &[T], size: isize, offset: GLintptr) {
        unsafe {
            gl::BufferSubData(
                self.target as GLenum,
                offset,
                size,
                data.as_ptr() as *const _,
            )
        };
    }

    pub fn bind(&mut self) {
        unsafe { gl::BindBuffer(self.target as GLenum, self.id) };
    }
    pub fn unbind(&mut self) {
        unsafe { gl::BindBuffer(self.target as GLenum, NULL_HANDLE) };
    }
}
