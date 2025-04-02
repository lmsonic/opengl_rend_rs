use std::ffi::CString;

use gl::types::GLsizei;
use glfw::{fail_on_errors, Window};
use glfw::{Action, Context, Key, Modifiers};
use opengl_rend::app::{run_app, Application};
use opengl_rend::buffer::{BufferType, Usage};
use opengl_rend::program::{Shader, ShaderType};
use opengl_rend::vertex_attributes::{DataType, VertexAttribute};
use opengl_rend::{
    buffer::Buffer, opengl::OpenGl, program::Program, vertex_attributes::VertexArrayObject,
};

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

impl Application for App {
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
            vertex_buffer, // needs to be around if not it gets dropped
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
    run_app::<App>();
}
