mod buffer;
mod opengl;
mod program;
mod vertex_attributes;

use std::ffi::CString;
use std::ptr;

use buffer::{Buffer, BufferType, Usage};
use gl::types::GLsizei;
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
    vertex_buffer: Buffer<f32>,
    vertex_array_object: VertexArrayObject,
}

const VERTEX_POSITIONS: [f32; 12] = [
    0.75, 0.75, 0.0, 1.0, 0.75, -0.75, 0.0, 1.0, -0.75, -0.75, 0.0, 1.0,
];

impl App {
    fn new(window: &mut Window) -> App {
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        let vert_str = CString::new(include_str!("vert.glsl")).unwrap();
        let frag_str = CString::new(include_str!("frag.glsl")).unwrap();
        let vert_shader = Shader::new(&vert_str, ShaderType::Vertex).unwrap();
        let frag_shader = Shader::new(&frag_str, ShaderType::Fragment).unwrap();
        let program = Program::new(&[vert_shader, frag_shader]).unwrap();
        let mut vertex_buffer = Buffer::new(BufferType::ArrayBuffer);
        vertex_buffer.buffer_data(&VERTEX_POSITIONS, Usage::StaticDraw);

        let mut vertex_array_object = VertexArrayObject::new();
        let vertex_attribute = VertexAttribute::new(4, DataType::Float, false);
        // vertex_buffer.bind();
        vertex_array_object.bind();
        vertex_array_object.set_attribute(0, &vertex_attribute, 0, 0);
        // vertex_buffer.unbind();

        unsafe { gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE) };

        Self {
            gl: OpenGl,
            program,
            vertex_buffer,
            vertex_array_object,
        }
    }

    fn display(&mut self) {
        self.gl.clear_color(0.1, 0.2, 0.3, 0.0);
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
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

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
