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

#[rustfmt::skip]
const VERTEX_DATA: [f32; 24] = [
    0.0, 0.5, 0.0, 1.0, 
    0.5, -0.366, 0.0, 1.0,
    -0.5, -0.366, 0.0, 1.0,
    1.0, 0.0, 0.0, 1.0,
    0.0, 1.0, 0.0, 1.0,
    0.0, 0.0, 1.0, 1.0,
];

impl App {
    fn new(window: &mut Window) -> App {
        let mut gl = OpenGl::new(window);
        // gl debug context
        gl.setup_debug_context();

        let vert_str = CString::new(include_str!("vert.vert")).unwrap();
        let frag_str = CString::new(include_str!("frag.frag")).unwrap();
        let vert_shader = Shader::new(&vert_str, ShaderType::Vertex).unwrap();
        let frag_shader = Shader::new(&frag_str, ShaderType::Fragment).unwrap();
        let program = Program::new(&[vert_shader, frag_shader]).unwrap();

        let mut vertex_buffer = Buffer::new(BufferType::ArrayBuffer);
        vertex_buffer.bind();
        vertex_buffer.buffer_data(&VERTEX_DATA, Usage::StaticDraw);

        let mut vertex_array_object = VertexArrayObject::new();
        let vec4 = VertexAttribute::new(4, DataType::Float, false);

        vertex_array_object.bind();
        vertex_array_object.set_attribute(0, &vec4, 0, 0);
        vertex_array_object.set_attribute(1, &vec4, 0, (vec4.size() * 3) as GLsizei);
        // gl.polygon_mode(opengl::PolygonMode::Line);
        Self {
            gl,
            program,
            vertex_array_object,
            vertex_buffer,
        }
    }

    fn display(&mut self) {
        self.gl.clear_color(0.5, 0.5, 0.5, 0.0);
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
