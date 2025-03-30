use gl::types::{GLbitfield, GLenum, GLfloat, GLint, GLsizei};
type GLHandle = gl::types::GLuint;
#[derive(Default)]
pub struct OpenGl {}

impl OpenGl {
    pub fn clear_color(&mut self, red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) {
        unsafe { gl::ClearColor(red, green, blue, alpha) };
    }
    pub fn clear(&mut self, mask: GLbitfield) {
        unsafe { gl::Clear(mask) };
    }
    pub fn draw_arrays(&mut self, mode: GLenum, first: GLint, count: GLsizei) {
        unsafe { gl::DrawArrays(gl::TRIANGLES, 0, 3) };
    }
}
