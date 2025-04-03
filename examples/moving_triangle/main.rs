use std::f32::consts::TAU;
use std::ffi::CString;

use gl::types::GLsizei;
use glfw::{Action, Key, Modifiers, PWindow};
use opengl_rend::app::{run_app, Application};
use opengl_rend::buffer::{BufferType, Usage};
use opengl_rend::program::{GLLocation, Shader, ShaderType};
use opengl_rend::vertex_attributes::{DataType, VertexAttribute};
use opengl_rend::{
    buffer::Buffer, opengl::OpenGl, program::Program, vertex_attributes::VertexArrayObject,
};

struct App {
    window: PWindow,
    gl: OpenGl,
    program: Program,
    vertex_array_object: VertexArrayObject,
    vertex_buffer: Buffer<f32>,
    elapsed_time_location: GLLocation,
}

#[rustfmt::skip]
const VERTEX_DATA: [f32;24] = [
    0.0, 0.5, 0.0, 1.0, 
    0.5, -0.366, 0.0, 1.0,
    -0.5, -0.366, 0.0, 1.0,
    1.0, 0.0, 0.0, 1.0,
    0.0, 1.0, 0.0, 1.0,
    0.0, 0.0, 1.0, 1.0,
    ];

impl App {
    fn _compute_position_offsets(&self, loop_duration: f32) -> (f32, f32) {
        let scale = TAU / loop_duration;

        let elapsed_time = self.window.glfw.get_time() as f32;
        let loop_time = elapsed_time % loop_duration;
        let (x_offset, y_offset) = (loop_time * scale).sin_cos();
        (x_offset * 0.5, y_offset * 0.5)
    }
    fn _adjust_vertex_data(&mut self, x_offset: f32, y_offset: f32) {
        let mut vertices = VERTEX_DATA;
        for i in (0..vertices.len()).step_by(4) {
            vertices[i] += x_offset;
            if i + 1 < vertices.len() {
                vertices[i + 1] += y_offset;
            }
        }
        self.vertex_buffer.bind();
        self.vertex_buffer.update_data(&vertices, 0);
        self.vertex_buffer.unbind();
    }
}

impl Application for App {
    fn new(mut window: PWindow) -> App {
        let gl = OpenGl::new(&mut window);

        let vert_str = CString::new(include_str!("vert.vert")).unwrap();
        let frag_str = CString::new(include_str!("frag.frag")).unwrap();
        let vert_shader = Shader::new(&vert_str, ShaderType::Vertex).unwrap();
        let frag_shader = Shader::new(&frag_str, ShaderType::Fragment).unwrap();
        let mut program = Program::new(&[vert_shader, frag_shader]).unwrap();

        let mut vertex_buffer = Buffer::new(BufferType::ArrayBuffer);
        vertex_buffer.bind();
        vertex_buffer.buffer_data(&VERTEX_DATA, Usage::StreamDraw);

        let mut vertex_array_object = VertexArrayObject::new();

        let vec4 = VertexAttribute::new(4, DataType::Float, false);

        vertex_array_object.bind();
        vertex_array_object.set_attribute(0, &vec4, 0, 0);
        vertex_array_object.set_attribute(1, &vec4, 0, (vec4.size() * 3) as GLsizei);
        // gl.polygon_mode(opengl::PolygonMode::Line);

        let loop_duration_location = program.get_uniform_location(c"loopDuration").unwrap();
        let frag_loop_duration_location =
            program.get_uniform_location(c"fragLoopDuration").unwrap();
        program.set_used();
        program.set_uniform(loop_duration_location, 5.0);
        program.set_uniform(frag_loop_duration_location, 10.0);
        program.set_unused();
        let elapsed_time_location = program.get_uniform_location(c"time").unwrap();
        Self {
            gl,
            program,
            vertex_array_object,
            vertex_buffer, // needs to be around if not it gets dropped
            window,
            elapsed_time_location,
        }
    }

    fn display(&mut self) {
        self.gl.clear_color(0.5, 0.5, 0.5, 0.0);
        self.gl.clear(gl::COLOR_BUFFER_BIT);

        self.program.set_used();
        let time = self.window.glfw.get_time() as f32;
        self.program.set_uniform(self.elapsed_time_location, time);

        self.vertex_buffer.bind();
        self.vertex_array_object.bind();

        self.gl.draw_arrays(gl::TRIANGLES, 0, 3);

        self.vertex_array_object.unbind();
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
