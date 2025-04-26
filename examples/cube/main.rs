#![forbid(unsafe_code)]

use std::ffi::CString;

use gl::types::GLsizei;
use glfw::{Action, Key, Modifiers, PWindow};
use opengl_rend::app::{run_app, Application};
use opengl_rend::buffer::{Target, Usage};
use opengl_rend::opengl::{Capability, ClearFlags, CullMode, Primitive, FrontFace};
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
    offset_location: GLLocation,
    perspective_matrix_location: GLLocation,
    perspective_matrix: [f32; 16],
    // perspective_matrix: Mat4,
}

#[rustfmt::skip]
const VERTEX_DATA: [f32;288] = [
    0.25,  0.25, -1.25, 1.0,
    0.25, -0.25, -1.25, 1.0,
   -0.25,  0.25, -1.25, 1.0,

    0.25, -0.25, -1.25, 1.0,
   -0.25, -0.25, -1.25, 1.0,
   -0.25,  0.25, -1.25, 1.0,

    0.25,  0.25, -2.75, 1.0,
   -0.25,  0.25, -2.75, 1.0,
    0.25, -0.25, -2.75, 1.0,

    0.25, -0.25, -2.75, 1.0,
   -0.25,  0.25, -2.75, 1.0,
   -0.25, -0.25, -2.75, 1.0,

   -0.25,  0.25, -1.25, 1.0,
   -0.25, -0.25, -1.25, 1.0,
   -0.25, -0.25, -2.75, 1.0,

   -0.25,  0.25, -1.25, 1.0,
   -0.25, -0.25, -2.75, 1.0,
   -0.25,  0.25, -2.75, 1.0,

    0.25,  0.25, -1.25, 1.0,
    0.25, -0.25, -2.75, 1.0,
    0.25, -0.25, -1.25, 1.0,

    0.25,  0.25, -1.25, 1.0,
    0.25,  0.25, -2.75, 1.0,
    0.25, -0.25, -2.75, 1.0,

    0.25,  0.25, -2.75, 1.0,
    0.25,  0.25, -1.25, 1.0,
   -0.25,  0.25, -1.25, 1.0,

    0.25,  0.25, -2.75, 1.0,
   -0.25,  0.25, -1.25, 1.0,
   -0.25,  0.25, -2.75, 1.0,

    0.25, -0.25, -2.75, 1.0,
   -0.25, -0.25, -1.25, 1.0,
    0.25, -0.25, -1.25, 1.0,

    0.25, -0.25, -2.75, 1.0,
   -0.25, -0.25, -2.75, 1.0,
   -0.25, -0.25, -1.25, 1.0,




   0.0, 0.0, 1.0, 1.0,
   0.0, 0.0, 1.0, 1.0,
   0.0, 0.0, 1.0, 1.0,

   0.0, 0.0, 1.0, 1.0,
   0.0, 0.0, 1.0, 1.0,
   0.0, 0.0, 1.0, 1.0,

   0.8, 0.8, 0.8, 1.0,
   0.8, 0.8, 0.8, 1.0,
   0.8, 0.8, 0.8, 1.0,

   0.8, 0.8, 0.8, 1.0,
   0.8, 0.8, 0.8, 1.0,
   0.8, 0.8, 0.8, 1.0,

   0.0, 1.0, 0.0, 1.0,
   0.0, 1.0, 0.0, 1.0,
   0.0, 1.0, 0.0, 1.0,

   0.0, 1.0, 0.0, 1.0,
   0.0, 1.0, 0.0, 1.0,
   0.0, 1.0, 0.0, 1.0,

   0.5, 0.5, 0.0, 1.0,
   0.5, 0.5, 0.0, 1.0,
   0.5, 0.5, 0.0, 1.0,

   0.5, 0.5, 0.0, 1.0,
   0.5, 0.5, 0.0, 1.0,
   0.5, 0.5, 0.0, 1.0,

   1.0, 0.0, 0.0, 1.0,
   1.0, 0.0, 0.0, 1.0,
   1.0, 0.0, 0.0, 1.0,

   1.0, 0.0, 0.0, 1.0,
   1.0, 0.0, 0.0, 1.0,
   1.0, 0.0, 0.0, 1.0,

   0.0, 1.0, 1.0, 1.0,
   0.0, 1.0, 1.0, 1.0,
   0.0, 1.0, 1.0, 1.0,

   0.0, 1.0, 1.0, 1.0,
   0.0, 1.0, 1.0, 1.0,
   0.0, 1.0, 1.0, 1.0,
];

impl Application for App {
    fn new(mut window: PWindow) -> App {
        let mut gl = OpenGl::new(&mut window);

        // initialize program
        let vert_str = CString::new(include_str!("vert.vert")).unwrap();
        let frag_str = CString::new(include_str!("frag.frag")).unwrap();
        let vert_shader = Shader::new(&vert_str, ShaderType::Vertex).unwrap();
        let frag_shader = Shader::new(&frag_str, ShaderType::Fragment).unwrap();
        let mut program = Program::new(&[vert_shader, frag_shader]).unwrap();

        // initialize vertex buffer
        let mut vertex_buffer = Buffer::new(Target::ArrayBuffer);
        vertex_buffer.bind();
        vertex_buffer.buffer_data(&VERTEX_DATA, Usage::StreamDraw);

        // initialize vao
        let mut vertex_array_object = VertexArrayObject::new();
        let vec4 = VertexAttribute::new(4, DataType::Float, false);

        let begin_color_data = std::mem::size_of_val(&VERTEX_DATA) / 2;

        vertex_array_object.bind();
        vertex_array_object.set_attribute(0, &vec4, 0, 0);
        vertex_array_object.set_attribute(1, &vec4, 0, begin_color_data as GLsizei);
        // gl.polygon_mode(PolygonMode::Line);

        gl.enable(Capability::CullFace);
        gl.cull_face(CullMode::Back);
        gl.front_face(FrontFace::CW);

        let offset_location = program.get_uniform_location(c"offset").unwrap();

        let frustum_scale = 1.0;
        let z_near = 1.0;
        let z_far = 3.0;

        let mut matrix: [f32; 16] = [0.0; 16];
        matrix[0] = frustum_scale;
        matrix[5] = frustum_scale;
        matrix[10] = (z_far + z_near) / (z_near - z_far);
        matrix[14] = (2.0 * z_far * z_near) / (z_near - z_far);
        matrix[11] = -1.0;

        // let fov = f32::to_radians(90.0);
        // let matrix = Mat4::perspective_rh_gl(fov, 1.0, z_near, z_far);

        let perspective_matrix_location =
            program.get_uniform_location(c"perspectiveMatrix").unwrap();

        program.set_used();
        program.set_uniform(perspective_matrix_location, matrix);
        program.set_unused();

        Self {
            gl,
            program,
            vertex_array_object,
            vertex_buffer, // needs to be around if not it gets dropped
            window,
            offset_location,
            perspective_matrix_location,
            perspective_matrix: matrix,
        }
    }

    fn display(&mut self) {
        self.gl.clear_color(0.5, 0.5, 0.5, 0.0);
        self.gl.clear(ClearFlags::Color);

        self.program.set_used();
        self.program.set_uniform(self.offset_location, (0.5, 0.5));
        self.vertex_buffer.bind();
        self.vertex_array_object.bind();

        self.gl.draw_arrays(Primitive::Triangles, 0, 36);

        self.vertex_array_object.unbind();
        self.program.set_unused();
    }

    fn keyboard(&mut self, _key: Key, _action: Action, _modifier: Modifiers) {}

    fn reshape(&mut self, width: i32, height: i32) {
        let frustum_scale = 1.0;

        self.perspective_matrix[0] = frustum_scale / (width as f32 / height as f32);
        self.perspective_matrix[5] = frustum_scale;

        // let z_near = 1.0;
        // let z_far = 3.0;
        // let fov = f32::to_radians(90.0);
        // self.perspective_matrix =
        //     Mat4::perspective_rh_gl(fov, width as f32 / height as f32, z_near, z_far);

        self.program.set_used();
        self.program
            .set_uniform(self.perspective_matrix_location, self.perspective_matrix);
        self.program.set_unused();

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
