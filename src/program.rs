use std::{
    ffi::{CStr, CString},
    ptr,
};

use gl::types::{GLenum, GLint, GLuint};

use crate::{uniforms::SetUniform, GLHandle};

pub type GLLocation = GLint;
pub type GLBlockIndex = GLuint;

pub struct Program {
    id: GLHandle,
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id) }
    }
}

impl Program {
    pub fn new(shaders: &[Shader]) -> Result<Self, CString> {
        let id = unsafe { gl::CreateProgram() };
        for shader in shaders {
            unsafe { gl::AttachShader(id, shader.id) };
        }
        unsafe { gl::LinkProgram(id) };
        let mut program = Self { id };
        if let Some(error) = program.get_link_error() {
            return Err(error);
        }
        for shader in shaders {
            unsafe { gl::DetachShader(id, shader.id) };
        }
        Ok(program)
    }

    fn get_link_error(&mut self) -> Option<CString> {
        let mut success = 0;
        unsafe { gl::GetProgramiv(self.id, gl::LINK_STATUS, &mut success) };
        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut len);
            }
            // convert buffer to CString
            let error: CString = create_whitespace_cstring_with_len(len as usize);
            unsafe {
                gl::GetProgramInfoLog(
                    self.id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr().cast_mut(),
                );
            }
            return Some(error);
        }
        None
    }

    pub fn set_used(&mut self) {
        unsafe { gl::UseProgram(self.id) };
    }
    pub fn set_unused(&mut self) {
        unsafe { gl::UseProgram(0) };
    }

    pub fn get_uniform_location(&mut self, name: &CStr) -> Option<GLLocation> {
        let loc = unsafe { gl::GetUniformLocation(self.id, name.as_ptr()) };
        if loc == -1 {
            return None;
        }
        Some(loc)
    }
    pub fn get_uniform_block_index(&mut self, name: &CStr) -> Option<GLBlockIndex> {
        let loc = unsafe { gl::GetUniformBlockIndex(self.id, name.as_ptr()) };
        if loc == gl::INVALID_INDEX {
            return None;
        }
        Some(loc)
    }
    pub fn uniform_block_binding(&mut self, block_index: GLBlockIndex, binding_index: GLuint) {
        unsafe { gl::UniformBlockBinding(self.id, block_index, binding_index) };
    }

    #[allow(private_bounds)]
    pub fn set_uniform<T: SetUniform>(&mut self, location: GLint, value: T) {
        value.set_uniform(location);
    }
}

pub struct Shader {
    id: GLHandle,
}

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum ShaderType {
    Compute = gl::COMPUTE_SHADER,
    Vertex = gl::VERTEX_SHADER,
    TessControl = gl::TESS_CONTROL_SHADER,
    TessEvaluation = gl::TESS_EVALUATION_SHADER,
    Geometry = gl::GEOMETRY_SHADER,
    Fragment = gl::FRAGMENT_SHADER,
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id) }
    }
}

impl Shader {
    fn get_compile_error(&mut self) -> Option<CString> {
        let mut success = 0;
        unsafe { gl::GetShaderiv(self.id, gl::COMPILE_STATUS, &mut success) };
        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetShaderiv(self.id, gl::INFO_LOG_LENGTH, &mut len);
            }
            // convert buffer to CString
            let error = create_whitespace_cstring_with_len(len as usize);
            unsafe {
                gl::GetShaderInfoLog(
                    self.id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr().cast_mut(),
                );
            }
            return Some(error);
        }
        None
    }
    pub fn new(text: &CStr, shader_type: ShaderType) -> Result<Self, CString> {
        let id = unsafe { gl::CreateShader(shader_type as GLenum) };
        let mut shader = Self { id };

        unsafe { gl::ShaderSource(shader.id, 1, &text.as_ptr(), ptr::null()) };

        unsafe { gl::CompileShader(shader.id) };

        if let Some(error) = shader.get_compile_error() {
            eprintln!("Compile failure in {shader_type:?} shader:\n{:?}", error);
            return Err(error);
        }
        Ok(shader)
    }
}
fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend(std::iter::once(&b' ').cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}
