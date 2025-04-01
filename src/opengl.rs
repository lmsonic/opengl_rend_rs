use std::{
    ffi::{c_void, CStr},
    ptr,
};

use gl::types::{GLbitfield, GLchar, GLenum, GLfloat, GLint, GLsizei, GLuint};
use glfw::Window;
pub struct OpenGl;

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum PolygonMode {
    Point = gl::POINT,
    Line = gl::LINE,
    Fill = gl::FILL,
}
extern "system" fn gl_debug_output(
    source: GLenum,
    type_: GLenum,
    id: GLuint,
    severity: GLenum,
    _length: GLsizei,
    message: *const GLchar,
    user_param: *mut c_void,
) {
    if id == 131169 || id == 131185 || id == 131218 || id == 131204 {
        return;
    }
    let message = unsafe { CStr::from_ptr(message) }.to_string_lossy();

    println!("------------");
    println!("Debug message ({id}) : {message:?} ");

    match source {
        gl::DEBUG_SOURCE_API => println!("Source: API"),
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => println!("Source: Window System"),
        gl::DEBUG_SOURCE_SHADER_COMPILER => println!("Source: Shader Compiler"),
        gl::DEBUG_SOURCE_THIRD_PARTY => println!("Source: Third Party"),
        gl::DEBUG_SOURCE_APPLICATION => println!("Source: Application"),
        gl::DEBUG_SOURCE_OTHER => println!("Source: Other"),
        _ => {}
    }
    match type_ {
        gl::DEBUG_TYPE_ERROR => println!("Type: Error"),
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => println!("Type: Deprecated Behaviour"),
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => println!("Type: Undefined Behaviour"),
        gl::DEBUG_TYPE_PORTABILITY => println!("Type: Portability"),
        gl::DEBUG_TYPE_PERFORMANCE => println!("Type: Performance"),
        gl::DEBUG_TYPE_MARKER => println!("Type: Marker"),
        gl::DEBUG_TYPE_PUSH_GROUP => println!("Type: Push Group"),
        gl::DEBUG_TYPE_POP_GROUP => println!("Type: Pop Group"),
        gl::DEBUG_TYPE_OTHER => println!("Type: Other"),
        _ => {}
    }
    match severity {
        gl::DEBUG_SEVERITY_HIGH => println!("Severity: high"),
        gl::DEBUG_SEVERITY_MEDIUM => println!("Severity: medium"),
        gl::DEBUG_SEVERITY_LOW => println!("Severity: low"),
        gl::DEBUG_SEVERITY_NOTIFICATION => println!("Severity: notification"),
        _ => {}
    }
}

impl OpenGl {
    pub fn setup_debug_context(&mut self) {
        let mut flags = 0;
        unsafe { gl::GetIntegerv(gl::CONTEXT_FLAGS, &mut flags) };
        if (flags as GLenum & gl::CONTEXT_FLAG_DEBUG_BIT) != 0 {
            // initialize debug output
            unsafe { gl::Enable(gl::DEBUG_OUTPUT) };
            unsafe { gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS) };
            unsafe { gl::DebugMessageCallback(Some(gl_debug_output), ptr::null()) }
            unsafe {
                gl::DebugMessageControl(
                    gl::DONT_CARE,
                    gl::DONT_CARE,
                    gl::DONT_CARE,
                    0,
                    ptr::null(),
                    gl::TRUE,
                )
            };
        }
    }

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

    pub fn new(window: &mut Window) -> Self {
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        OpenGl
    }
}
