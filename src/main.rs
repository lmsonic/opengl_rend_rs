mod buffer;
mod opengl;
mod program;
mod vertex_attributes;

use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::ptr;

use buffer::{Buffer, BufferType, Usage};
use gl::types::{GLchar, GLenum, GLsizei, GLuint};
use glfw::{fail_on_errors, Window};
use glfw::{Action, Context, Key, Modifiers};
use opengl::OpenGl;
use program::{Program, Shader, ShaderType};
use vertex_attributes::{DataType, VertexArrayObject, VertexAttribute};
const NULL_HANDLE: GLHandle = 0;

type GLHandle = gl::types::GLuint;

struct App {
    gl: OpenGl,
    program: Program,
    vertex_array_object: VertexArrayObject,
    vertex_buffer: Buffer<f32>,
}

const VERTEX_POSITIONS: [f32; 12] = [
    0.75, 0.75, 0.0, 1.0, 0.75, -0.75, 0.0, 1.0, -0.75, -0.75, 0.0, 1.0,
];

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

impl App {
    fn new(window: &mut Window) -> App {
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        let gl = OpenGl;
        // gl debug context
        setup_opengl_debug_context();

        let vert_str = CString::new(include_str!("vert.vert")).unwrap();
        let frag_str = CString::new(include_str!("frag.frag")).unwrap();
        let vert_shader = Shader::new(&vert_str, ShaderType::Vertex).unwrap();
        let frag_shader = Shader::new(&frag_str, ShaderType::Fragment).unwrap();
        let program = Program::new(&[vert_shader, frag_shader]).unwrap();

        let mut vertex_buffer = Buffer::new(BufferType::ArrayBuffer);
        vertex_buffer.bind();
        vertex_buffer.buffer_data(&VERTEX_POSITIONS, Usage::StaticDraw);

        let mut vertex_array_object = VertexArrayObject::new();
        let vertex_attribute = VertexAttribute::new(4, DataType::Float, false);
        vertex_array_object.bind();
        vertex_array_object.set_attribute(0, &vertex_attribute, 0, 0);
        // gl.polygon_mode(opengl::PolygonMode::Line);
        Self {
            gl,
            program,
            vertex_array_object,
            vertex_buffer,
        }
    }

    fn display(&mut self) {
        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.gl.clear(gl::COLOR_BUFFER_BIT);

        self.program.set_used();
        self.vertex_array_object.bind();

        self.gl.draw_arrays(gl::TRIANGLES, 0, 3);

        self.program.set_unused();
    }

    fn keyboard(&mut self, key: Key, action: Action, modifier: Modifiers) {}

    fn reshape(&mut self, width: i32, height: i32) {
        self.gl.viewport(0, 0, width as GLsizei, height as GLsizei);
    }
}

fn setup_opengl_debug_context() {
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

fn main() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));

    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw
        .create_window(600, 600, "OpenGl", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    // Make the window's context current
    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    let mut app = App::new(&mut window);

    // Loop until the user closes the window
    while !window.should_close() {
        // process events
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                glfw::WindowEvent::Key(key, _, action, modifier) => {
                    app.keyboard(key, action, modifier)
                }

                glfw::WindowEvent::FramebufferSize(width, height) => app.reshape(width, height),
                _ => {}
            }
        }

        // render
        app.display();

        // Swap front and back buffers
        window.swap_buffers();

        // Poll for and process events
        glfw.poll_events();
    }
}
