#![forbid(unsafe_code)]
use std::f32::consts::TAU;
use std::ffi::CString;

use gl::types::GLsizei;
use glfw::{Action, Key, Modifiers, PWindow};
use opengl_rend::app::{run_app, Application};
use opengl_rend::buffer::{BufferType, Usage};
use opengl_rend::opengl::{
    Capability, ClearFlags, CullMode, DepthFunc, DrawMode, FrontFace, IndexSize,
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
const BROWN_COLOR: [f32; 4] = [0.5, 0.5, 0.0, 1.0];

const NUMBER_OF_VERTICES: usize = 8;

#[rustfmt::skip]
const VERTEX_DATA: [f32;56] = [
    1.0, 1.0, 1.0,
    -1.0, -1.0, 1.0,
    -1.0, 1.0, -1.0,
    1.0, -1.0, -1.0,

    -1.0, -1.0, -1.0,
    1.0, 1.0, -1.0,
    1.0, -1.0, 1.0,
    -1.0, 1.0, 1.0,

    GREEN_COLOR[0],GREEN_COLOR[1],GREEN_COLOR[2],GREEN_COLOR[3],
    BLUE_COLOR[0],BLUE_COLOR[1],BLUE_COLOR[2],BLUE_COLOR[3],
    RED_COLOR[0],RED_COLOR[1],RED_COLOR[2],RED_COLOR[3],
    BROWN_COLOR[0],BROWN_COLOR[1],BROWN_COLOR[2],BROWN_COLOR[3],

    GREEN_COLOR[0],GREEN_COLOR[1],GREEN_COLOR[2],GREEN_COLOR[3],
    BLUE_COLOR[0],BLUE_COLOR[1],BLUE_COLOR[2],BLUE_COLOR[3],
    RED_COLOR[0],RED_COLOR[1],RED_COLOR[2],RED_COLOR[3],
    BROWN_COLOR[0],BROWN_COLOR[1],BROWN_COLOR[2],BROWN_COLOR[3],

];

#[rustfmt::skip]
const INDEX_DATA: [u32;24] =[
	0, 1, 2,
	1, 0, 3,
	2, 3, 0,
	3, 2, 1,

	5, 4, 6,
	4, 5, 7,
	7, 6, 4,
	6, 7, 5,
];

#[derive(Clone, Copy)]
enum OffsetFunc {
    Stationary,
    Oval,
    BottomCircle,
}

impl OffsetFunc {
    fn calc_offset(self, elapsed_time: f32) -> glam::Vec3 {
        match self {
            OffsetFunc::Stationary => glam::Vec3::new(0.0, 0.0, -20.0),
            OffsetFunc::Oval => {
                const LOOP_DURATION: f32 = 3.0;
                const SCALE: f32 = TAU / LOOP_DURATION;
                let current_time = elapsed_time % LOOP_DURATION;
                glam::Vec3::new(
                    f32::cos(current_time * SCALE) * 4.0,
                    f32::sin(current_time * SCALE) * 6.0,
                    -20.0,
                )
            }
            OffsetFunc::BottomCircle => {
                const LOOP_DURATION: f32 = 12.0;
                const SCALE: f32 = TAU / LOOP_DURATION;
                let current_time = elapsed_time % LOOP_DURATION;
                glam::Vec3::new(
                    f32::cos(current_time * SCALE) * 5.0,
                    -3.5,
                    f32::sin(current_time * SCALE) * 5.0 - 20.0,
                )
            }
        }
    }
    fn offset_matrix(self, elapsed: f32) -> glam::Mat4 {
        glam::Mat4::from_translation(self.calc_offset(elapsed))
    }
}
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
        let mut vertex_buffer_object = VertexArrayObject::new();
        vertex_buffer_object.bind();
        let vec3 = VertexAttribute::new(3, DataType::Float, false);
        let vec4 = VertexAttribute::new(4, DataType::Float, false);

        let color_data_offset = std::mem::size_of::<f32>() * 3 * NUMBER_OF_VERTICES;

        vertex_buffer.bind();
        vertex_buffer_object.set_attribute(0, &vec3, 0, 0);
        vertex_buffer_object.set_attribute(1, &vec4, 0, color_data_offset as GLsizei);
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
        let z_far = 45.0;

        let mut matrix: [f32; 16] = [0.0; 16];
        matrix[0] = frustum_scale;
        matrix[5] = frustum_scale;
        matrix[10] = (z_far + z_near) / (z_near - z_far);
        matrix[14] = (2.0 * z_far * z_near) / (z_near - z_far);
        matrix[11] = -1.0;

        program.set_used();
        program.set_uniform(
            model_to_camera_matrix_location,
            OffsetFunc::Stationary.offset_matrix(0.0),
        );
        program.set_uniform(camera_to_clip_location, matrix);
        program.set_unused();

        Self {
            gl,
            program,
            vertex_array_object: vertex_buffer_object,
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
        let elapsed = self.window.glfw.get_time() as f32;
        let matrices = [
            OffsetFunc::Stationary.offset_matrix(elapsed),
            OffsetFunc::Oval.offset_matrix(elapsed),
            OffsetFunc::BottomCircle.offset_matrix(elapsed),
        ];
        for m in matrices {
            self.program
                .set_uniform(self.model_to_camera_matrix_location, m);
            self.gl.draw_elements(
                DrawMode::Triangles,
                INDEX_DATA.len() as i32,
                IndexSize::UnsignedInt,
                0,
            );
        }

        self.vertex_array_object.unbind();
        self.program.set_unused();
    }

    fn keyboard(&mut self, _key: Key, _action: Action, _modifier: Modifiers) {}

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
