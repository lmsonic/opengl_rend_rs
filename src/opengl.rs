use gl::types::{GLbitfield, GLenum, GLfloat, GLint, GLsizei};
pub struct OpenGl;

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum PolygonMode {
    Point = gl::POINT,
    Line = gl::LINE,
    Fill = gl::FILL,
}

impl OpenGl {
    pub fn clear_color(&mut self, red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) {
        unsafe { gl::ClearColor(red, green, blue, alpha) };
    }
    pub fn clear(&mut self, mask: GLbitfield) {
        unsafe { gl::Clear(mask) };
    }
    pub fn draw_arrays(&mut self, mode: GLenum, first: GLint, count: GLsizei) {
        unsafe { gl::DrawArrays(mode, first, count) };
    }

    pub fn viewport(&mut self, x: GLsizei, y: GLsizei, width: GLsizei, height: GLsizei) {
        unsafe {
            gl::Viewport(x, y, width, height);
        }
    }
    pub fn polygon_mode(&mut self, mode: PolygonMode) {
        unsafe { gl::PolygonMode(gl::FRONT_AND_BACK, mode as GLenum) };
    }
}
