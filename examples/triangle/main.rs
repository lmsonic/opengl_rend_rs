#![forbid(unsafe_code)]

use std::ffi::CString;

use gl::types::GLsizei;
use glfw::PWindow;
use glfw::{Action, Key, Modifiers};
use opengl_rend::app::{run_app, Application};
use opengl_rend::buffer::{Target, Usage};
use opengl_rend::opengl::{ClearFlags, Primitive};
use opengl_rend::program::{Shader, ShaderType};
use opengl_rend::vertex_attributes::{DataType, VertexAttribute};
use opengl_rend::{
    buffer::Buffer, opengl::OpenGl, program::Program, vertex_attributes::VertexArrayObject,
};

struct App {
    window: PWindow,
    gl: OpenGl,
    program: Program,
    vertex_array_object: VertexArrayObject,
    _vertex_buffer: Buffer<f32>,
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
    fn new(mut window: PWindow) -> Self {
        let gl = OpenGl::new(&mut window);

        let vert_str = CString::new(include_str!("vert.vert")).unwrap();
        let frag_str = CString::new(include_str!("frag.frag")).unwrap();
        let vert_shader = Shader::new(&vert_str, ShaderType::Vertex).unwrap();
        let frag_shader = Shader::new(&frag_str, ShaderType::Fragment).unwrap();
        let program = Program::new(&[vert_shader, frag_shader]).unwrap();

        let mut vertex_buffer = Buffer::new(Target::ArrayBuffer);
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
            _vertex_buffer: vertex_buffer, // needs to be kept around if not it gets dropped
            window,
        }
    }

    fn display(&mut self) {
        self.gl.clear_color(0.5, 0.5, 0.5, 0.0);
        self.gl.clear(ClearFlags::Color);

        self.program.set_used();
        self.vertex_array_object.bind();

        self.gl.draw_arrays(Primitive::Triangles, 0, 3);

        self.program.set_unused();
    }

    fn keyboard(&mut self, _key: Key, _action: Action, _modifier: Modifiers) {}

    fn reshape(&mut self, width: i32, height: i32) {
        self.gl.viewport(0, 0, width as GLsizei, height as GLsizei);
    }

    fn window(&self) -> &PWindow {
        &self.window
    }

    fn window_mut(&mut self) -> &mut PWindow {
        &mut self.window
    }
}

fn main() {
    run_app::<App>();
}
