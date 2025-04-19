#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
pub mod app;
pub mod buffer;
pub mod matrix_stack;
pub mod mesh;
pub mod opengl;
pub mod program;
pub mod uniforms;
pub mod vertex_attributes;

const NULL_HANDLE: GLHandle = 0;

type GLHandle = gl::types::GLuint;
