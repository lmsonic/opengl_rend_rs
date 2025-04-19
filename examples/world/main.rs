#![forbid(unsafe_code)]
use std::ffi::CString;

use gl::types::GLsizei;
use glam::{Mat4, Vec3};
use glfw::{Action, Key, Modifiers, PWindow};
use opengl_rend::app::{run_app, Application};
use opengl_rend::buffer::{BufferType, Usage};
use opengl_rend::opengl::{
    Capability, ClearFlags, CullMode, DepthFunc, FrontFace, IndexSize, Primitive,
};
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
    _vertex_buffer: Buffer<f32>,
    _index_buffer: Buffer<u32>,
    camera_to_clip_location: GLLocation,
    model_to_camera_matrix_location: GLLocation,
    perspective_matrix: [f32; 16],
    _depth_clamping: bool,
}

const GREEN_COLOR: [f32; 4] = [0.75, 0.75, 1.0, 1.0];
const BLUE_COLOR: [f32; 4] = [0.0, 0.5, 0.0, 1.0];
const RED_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const YELLOW_COLOR: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
const CYAN_COLOR: [f32; 4] = [0.0, 1.0, 1.0, 1.0];
const MAGENTA_COLOR: [f32; 4] = [1.0, 0.0, 1.0, 1.0];

const NUMBER_OF_VERTICES: usize = 24;

#[rustfmt::skip]
const VERTEX_DATA: [f32;168] = [
    //Front
    1.0, 1.0, 1.0,
    1.0, -1.0, 1.0,
    -1.0, -1.0, 1.0,
    -1.0, 1.0, 1.0,

    //Top
    1.0, 1.0, 1.0,
    -1.0, 1.0, 1.0,
    -1.0, 1.0, -1.0,
    1.0, 1.0, -1.0,

    //Let
    1.0, 1.0, 1.0,
    1.0, 1.0, -1.0,
    1.0, -1.0, -1.0,
    1.0, -1.0, 1.0,

    //Back
    1.0, 1.0, -1.0,
    -1.0, 1.0, -1.0,
    -1.0, -1.0, -1.0,
    1.0, -1.0, -1.0,

    //Bottom
    1.0, -1.0, 1.0,
    1.0, -1.0, -1.0,
    -1.0, -1.0, -1.0,
    -1.0, -1.0, 1.0,

    //Right
    -1.0, 1.0, 1.0,
    -1.0, -1.0, 1.0,
    -1.0, -1.0, -1.0,
    -1.0, 1.0, -1.0,

    GREEN_COLOR[0],GREEN_COLOR[1],GREEN_COLOR[2],GREEN_COLOR[3],
    GREEN_COLOR[0],GREEN_COLOR[1],GREEN_COLOR[2],GREEN_COLOR[3],
    GREEN_COLOR[0],GREEN_COLOR[1],GREEN_COLOR[2],GREEN_COLOR[3],
    GREEN_COLOR[0],GREEN_COLOR[1],GREEN_COLOR[2],GREEN_COLOR[3],

    BLUE_COLOR[0],BLUE_COLOR[1],BLUE_COLOR[2],BLUE_COLOR[3],
    BLUE_COLOR[0],BLUE_COLOR[1],BLUE_COLOR[2],BLUE_COLOR[3],
    BLUE_COLOR[0],BLUE_COLOR[1],BLUE_COLOR[2],BLUE_COLOR[3],
    BLUE_COLOR[0],BLUE_COLOR[1],BLUE_COLOR[2],BLUE_COLOR[3],

    RED_COLOR[0],RED_COLOR[1],RED_COLOR[2],RED_COLOR[3],
    RED_COLOR[0],RED_COLOR[1],RED_COLOR[2],RED_COLOR[3],
    RED_COLOR[0],RED_COLOR[1],RED_COLOR[2],RED_COLOR[3],
    RED_COLOR[0],RED_COLOR[1],RED_COLOR[2],RED_COLOR[3],

    YELLOW_COLOR[0],YELLOW_COLOR[1],YELLOW_COLOR[2],YELLOW_COLOR[3],
    YELLOW_COLOR[0],YELLOW_COLOR[1],YELLOW_COLOR[2],YELLOW_COLOR[3],
    YELLOW_COLOR[0],YELLOW_COLOR[1],YELLOW_COLOR[2],YELLOW_COLOR[3],
    YELLOW_COLOR[0],YELLOW_COLOR[1],YELLOW_COLOR[2],YELLOW_COLOR[3],

    CYAN_COLOR[0],CYAN_COLOR[1],CYAN_COLOR[2],CYAN_COLOR[3],
    CYAN_COLOR[0],CYAN_COLOR[1],CYAN_COLOR[2],CYAN_COLOR[3],
    CYAN_COLOR[0],CYAN_COLOR[1],CYAN_COLOR[2],CYAN_COLOR[3],
    CYAN_COLOR[0],CYAN_COLOR[1],CYAN_COLOR[2],CYAN_COLOR[3],
    
    MAGENTA_COLOR[0],MAGENTA_COLOR[1],MAGENTA_COLOR[2],MAGENTA_COLOR[3],
    MAGENTA_COLOR[0],MAGENTA_COLOR[1],MAGENTA_COLOR[2],MAGENTA_COLOR[3],
    MAGENTA_COLOR[0],MAGENTA_COLOR[1],MAGENTA_COLOR[2],MAGENTA_COLOR[3],
    MAGENTA_COLOR[0],MAGENTA_COLOR[1],MAGENTA_COLOR[2],MAGENTA_COLOR[3],
   

];

#[rustfmt::skip]
const INDEX_DATA: [u32;36] =[
	0, 1, 2,
	2, 3, 0,

	4, 5, 6,
	6, 7, 4,

	8, 9, 10,
	10, 11, 8,

	12, 13, 14,
	14, 15, 12,

	16, 17, 18,
	18, 19, 16,

	20, 21, 22,
	22, 23, 20,
];
fn calculate_frustum_scale(fov_degrees: f32) -> f32 {
    let fov_radians = fov_degrees.to_radians();
    (fov_radians * 0.5).tan().recip()
}

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
        let mut vertex_buffer = Buffer::new(BufferType::ArrayBuffer);
        vertex_buffer.bind();
        vertex_buffer.buffer_data(&VERTEX_DATA, Usage::StaticDraw);
        vertex_buffer.unbind();
        // initialize index buffer
        let mut index_buffer = Buffer::new(BufferType::IndexBuffer);
        index_buffer.bind();
        index_buffer.buffer_data(&INDEX_DATA, Usage::StaticDraw);
        // initialize vaos
        let mut vertex_array_object = VertexArrayObject::new();
        vertex_array_object.bind();
        let vec3 = VertexAttribute::new(3, DataType::Float, false);
        let vec4 = VertexAttribute::new(4, DataType::Float, false);

        let color_data_offset = std::mem::size_of::<f32>() * 3 * NUMBER_OF_VERTICES;

        vertex_buffer.bind();
        vertex_array_object.set_attribute(0, &vec3, 0, 0);
        vertex_array_object.set_attribute(1, &vec4, 0, color_data_offset as GLsizei);
        index_buffer.bind();

        // enable backface culling
        gl.enable(Capability::CullFace);
        gl.cull_face(CullMode::Back);
        gl.front_face(FrontFace::CW);
        // gl.polygon_mode(PolygonMode::Line);

        // enable depth test
        gl.enable(Capability::DepthTest);
        gl.set_depth_mask(true);
        gl.depth_func(DepthFunc::LessEqual);
        gl.depth_range(0.0, 1.0);

        // get and set uniforms
        let model_to_camera_matrix_location =
            program.get_uniform_location(c"modelToCamera").unwrap();
        let camera_to_clip_location = program.get_uniform_location(c"cameraToClip").unwrap();

        let frustum_scale = calculate_frustum_scale(45.0);
        let z_near = 1.0;
        let z_far = 100.0;

        let mut matrix: [f32; 16] = [0.0; 16];
        matrix[0] = frustum_scale;
        matrix[5] = frustum_scale;
        matrix[10] = (z_far + z_near) / (z_near - z_far);
        matrix[14] = (2.0 * z_far * z_near) / (z_near - z_far);
        matrix[11] = -1.0;

        program.set_used();
        program.set_uniform(camera_to_clip_location, matrix);
        program.set_unused();

        Self {
            gl,
            program,
            vertex_array_object,
            _vertex_buffer: vertex_buffer,
            _index_buffer: index_buffer,
            window,
            camera_to_clip_location,
            perspective_matrix: matrix,
            _depth_clamping: false,
            model_to_camera_matrix_location,
        }
    }

    fn display(&mut self) {
        self.gl.clear_color(0.1, 0.1, 0.1, 0.0);
        self.gl.clear_depth(1.0);
        self.gl.clear(ClearFlags::Color | ClearFlags::Depth);
        self.program.set_used();
        self.vertex_array_object.bind();

        self.vertex_array_object.unbind();
        self.program.set_unused();
    }

    fn keyboard(&mut self, key: Key, action: Action, _modifier: Modifiers) {
        if action == Action::Press || action == Action::Repeat {
            match key {
                _ => {}
            }
        }
    }

    fn reshape(&mut self, width: i32, height: i32) {
        let frustum_scale = calculate_frustum_scale(45.0);

        self.perspective_matrix[0] = frustum_scale / (width as f32 / height as f32);
        self.perspective_matrix[5] = frustum_scale;

        self.program.set_used();
        self.program
            .set_uniform(self.camera_to_clip_location, self.perspective_matrix);
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
