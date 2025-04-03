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
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum Capability {
    Blend = gl::BLEND,
    ClipDistance0 = gl::CLIP_DISTANCE0,
    ClipDistance1 = gl::CLIP_DISTANCE1,
    ClipDistance2 = gl::CLIP_DISTANCE2,
    ClipDistance3 = gl::CLIP_DISTANCE3,
    ClipDistance4 = gl::CLIP_DISTANCE4,
    ClipDistance5 = gl::CLIP_DISTANCE5,
    ClipDistance6 = gl::CLIP_DISTANCE6,
    ClipDistance7 = gl::CLIP_DISTANCE7,
    ColorLogicOp = gl::COLOR_LOGIC_OP,
    CullFace = gl::CULL_FACE,
    DebugOutput = gl::DEBUG_OUTPUT,
    DebugOutputSync = gl::DEBUG_OUTPUT_SYNCHRONOUS,
    DepthClamp = gl::DEPTH_CLAMP,
    DepthTest = gl::DEPTH_TEST,
    Dither = gl::DITHER,
    FramebufferSrgb = gl::FRAMEBUFFER_SRGB,
    LineSmooth = gl::LINE_SMOOTH,
    MULTISAMPLE = gl::MULTISAMPLE,
    PolygonOffsetFill = gl::POLYGON_OFFSET_FILL,
    PolygonOffsetLine = gl::POLYGON_OFFSET_LINE,
    PolygonSmooth = gl::POLYGON_SMOOTH,
    PrimitiveRestart = gl::PRIMITIVE_RESTART,
    PrimitiveRestartFixedIndex = gl::PRIMITIVE_RESTART_FIXED_INDEX,
    RasterizerDiscard = gl::RASTERIZER_DISCARD,
    SampleAlphaToCoverage = gl::SAMPLE_ALPHA_TO_COVERAGE,
    SampleAlphaToOne = gl::SAMPLE_ALPHA_TO_ONE,
    SampleCoverage = gl::SAMPLE_COVERAGE,
    SampleShading = gl::SAMPLE_SHADING,
    SampleMask = gl::SAMPLE_MASK,
    ScissorTest = gl::SCISSOR_TEST,
    StencilTest = gl::STENCIL_TEST,
    TextureCubeMapSeamless = gl::TEXTURE_CUBE_MAP_SEAMLESS,
    ProgramPointSize = gl::PROGRAM_POINT_SIZE,
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
    pub fn new(window: &mut Window) -> Self {
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        OpenGl
    }

    pub fn enable(&mut self, cap: Capability) {
        unsafe { gl::Enable(cap as GLenum) };
    }
    pub fn disable(&mut self, cap: Capability) {
        unsafe { gl::Disable(cap as GLenum) };
    }
    pub fn is_enabled(&mut self, cap: Capability) -> bool {
        if unsafe { gl::IsEnabled(cap as GLenum) } != gl::FALSE {
            return true;
        } else {
            false
        }
    }

    pub fn setup_debug_context(&mut self) {
        let mut flags = 0;
        unsafe { gl::GetIntegerv(gl::CONTEXT_FLAGS, &mut flags) };
        if (flags as GLenum & gl::CONTEXT_FLAG_DEBUG_BIT) != 0 {
            // initialize debug output
            self.enable(Capability::DebugOutput);
            self.enable(Capability::DebugOutputSync);
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
}
