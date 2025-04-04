use std::ffi::CString;

use gl::types::GLsizei;
use glfw::{Action, Key, Modifiers, PWindow};
use opengl_rend::app::{run_app, Application};
use opengl_rend::buffer::{BufferType, Usage};
use opengl_rend::opengl::{Capability, CullMode, DrawMode, FrontFace, IndexSize};
use opengl_rend::program::{GLLocation, Shader, ShaderType};
use opengl_rend::vertex_attributes::{DataType, VertexAttribute};
use opengl_rend::{
    buffer::Buffer, opengl::OpenGl, program::Program, vertex_attributes::VertexArrayObject,
};

struct App {
    window: PWindow,
    gl: OpenGl,
    program: Program,
    vertex_buffer_object: VertexArrayObject,
    vertex_buffer: Buffer<f32>,
    index_buffer: Buffer<u32>,
    offset_location: GLLocation,
    perspective_matrix_location: GLLocation,
    perspective_matrix: [f32; 16],
}

const RIGHT_EXTENT: f32 = 0.8;
const LEFT_EXTENT: f32 = -RIGHT_EXTENT;
const TOP_EXTENT: f32 = 0.20;
const MIDDLE_EXTENT: f32 = 0.0;
const BOTTOM_EXTENT: f32 = -TOP_EXTENT;
const FRONT_EXTENT: f32 = -1.25;
const REAR_EXTENT: f32 = -1.75;

const GREEN_COLOR: [f32; 4] = [0.75, 0.75, 1.0, 1.0];
const BLUE_COLOR: [f32; 4] = [0.0, 0.5, 0.0, 1.0];
const RED_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GREY_COLOR: [f32; 4] = [0.8, 0.8, 0.8, 1.0];
const BROWN_COLOR: [f32; 4] = [0.5, 0.5, 0.0, 1.0];

const NUMBER_OF_VERTICES: usize = 36;

#[rustfmt::skip]
const VERTEX_DATA: [f32;252] = [
//Object 1 positions
    LEFT_EXTENT,	TOP_EXTENT,		REAR_EXTENT,
    LEFT_EXTENT,	MIDDLE_EXTENT,	FRONT_EXTENT,
    RIGHT_EXTENT,	MIDDLE_EXTENT,	FRONT_EXTENT,
    RIGHT_EXTENT,	TOP_EXTENT,		REAR_EXTENT,

    LEFT_EXTENT,	BOTTOM_EXTENT,	REAR_EXTENT,
    LEFT_EXTENT,	MIDDLE_EXTENT,	FRONT_EXTENT,
    RIGHT_EXTENT,	MIDDLE_EXTENT,	FRONT_EXTENT,
    RIGHT_EXTENT,	BOTTOM_EXTENT,	REAR_EXTENT,

    LEFT_EXTENT,	TOP_EXTENT,		REAR_EXTENT,
    LEFT_EXTENT,	MIDDLE_EXTENT,	FRONT_EXTENT,
    LEFT_EXTENT,	BOTTOM_EXTENT,	REAR_EXTENT,

    RIGHT_EXTENT,	TOP_EXTENT,		REAR_EXTENT,
    RIGHT_EXTENT,	MIDDLE_EXTENT,	FRONT_EXTENT,
    RIGHT_EXTENT,	BOTTOM_EXTENT,	REAR_EXTENT,

    LEFT_EXTENT,	BOTTOM_EXTENT,	REAR_EXTENT,
    LEFT_EXTENT,	TOP_EXTENT,		REAR_EXTENT,
    RIGHT_EXTENT,	TOP_EXTENT,		REAR_EXTENT,
    RIGHT_EXTENT,	BOTTOM_EXTENT,	REAR_EXTENT,

    //Object 2 positions
    TOP_EXTENT,		RIGHT_EXTENT,	REAR_EXTENT,
    MIDDLE_EXTENT,	RIGHT_EXTENT,	FRONT_EXTENT,
    MIDDLE_EXTENT,	LEFT_EXTENT,	FRONT_EXTENT,
    TOP_EXTENT,		LEFT_EXTENT,	REAR_EXTENT,

    BOTTOM_EXTENT,	RIGHT_EXTENT,	REAR_EXTENT,
    MIDDLE_EXTENT,	RIGHT_EXTENT,	FRONT_EXTENT,
    MIDDLE_EXTENT,	LEFT_EXTENT,	FRONT_EXTENT,
    BOTTOM_EXTENT,	LEFT_EXTENT,	REAR_EXTENT,

    TOP_EXTENT,		RIGHT_EXTENT,	REAR_EXTENT,
    MIDDLE_EXTENT,	RIGHT_EXTENT,	FRONT_EXTENT,
    BOTTOM_EXTENT,	RIGHT_EXTENT,	REAR_EXTENT,
                    
    TOP_EXTENT,		LEFT_EXTENT,	REAR_EXTENT,
    MIDDLE_EXTENT,	LEFT_EXTENT,	FRONT_EXTENT,
    BOTTOM_EXTENT,	LEFT_EXTENT,	REAR_EXTENT,
                    
    BOTTOM_EXTENT,	RIGHT_EXTENT,	REAR_EXTENT,
    TOP_EXTENT,		RIGHT_EXTENT,	REAR_EXTENT,
    TOP_EXTENT,		LEFT_EXTENT,	REAR_EXTENT,
    BOTTOM_EXTENT,	LEFT_EXTENT,	REAR_EXTENT,

//Object 1 colors
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

    GREY_COLOR[0],GREY_COLOR[1],GREY_COLOR[2],GREY_COLOR[3],
    GREY_COLOR[0],GREY_COLOR[1],GREY_COLOR[2],GREY_COLOR[3],
    GREY_COLOR[0],GREY_COLOR[1],GREY_COLOR[2],GREY_COLOR[3],

    BROWN_COLOR[0],BROWN_COLOR[1],BROWN_COLOR[2],BROWN_COLOR[3],
    BROWN_COLOR[0],BROWN_COLOR[1],BROWN_COLOR[2],BROWN_COLOR[3],
    BROWN_COLOR[0],BROWN_COLOR[1],BROWN_COLOR[2],BROWN_COLOR[3],
    BROWN_COLOR[0],BROWN_COLOR[1],BROWN_COLOR[2],BROWN_COLOR[3],

    //Object 2 colors
    RED_COLOR[0],RED_COLOR[1],RED_COLOR[2],RED_COLOR[3],
    RED_COLOR[0],RED_COLOR[1],RED_COLOR[2],RED_COLOR[3],
    RED_COLOR[0],RED_COLOR[1],RED_COLOR[2],RED_COLOR[3],
    RED_COLOR[0],RED_COLOR[1],RED_COLOR[2],RED_COLOR[3],

    BROWN_COLOR[0],BROWN_COLOR[1],BROWN_COLOR[2],BROWN_COLOR[3],
    BROWN_COLOR[0],BROWN_COLOR[1],BROWN_COLOR[2],BROWN_COLOR[3],
    BROWN_COLOR[0],BROWN_COLOR[1],BROWN_COLOR[2],BROWN_COLOR[3],
    BROWN_COLOR[0],BROWN_COLOR[1],BROWN_COLOR[2],BROWN_COLOR[3],

    BLUE_COLOR[0],BLUE_COLOR[1],BLUE_COLOR[2],BLUE_COLOR[3],
    BLUE_COLOR[0],BLUE_COLOR[1],BLUE_COLOR[2],BLUE_COLOR[3],
    BLUE_COLOR[0],BLUE_COLOR[1],BLUE_COLOR[2],BLUE_COLOR[3],

    GREEN_COLOR[0],GREEN_COLOR[1],GREEN_COLOR[2],GREEN_COLOR[3],
    GREEN_COLOR[0],GREEN_COLOR[1],GREEN_COLOR[2],GREEN_COLOR[3],
    GREEN_COLOR[0],GREEN_COLOR[1],GREEN_COLOR[2],GREEN_COLOR[3],

    GREY_COLOR[0],GREY_COLOR[1],GREY_COLOR[2],GREY_COLOR[3],
    GREY_COLOR[0],GREY_COLOR[1],GREY_COLOR[2],GREY_COLOR[3],
    GREY_COLOR[0],GREY_COLOR[1],GREY_COLOR[2],GREY_COLOR[3],
    GREY_COLOR[0],GREY_COLOR[1],GREY_COLOR[2],GREY_COLOR[3],
];

#[rustfmt::skip]
const INDEX_DATA: [u32;24] =[
    0, 2, 1,
	3, 2, 0,

	4, 5, 6,
	6, 7, 4,

	8, 9, 10,
	11, 13, 12,

	14, 16, 15,
	17, 16, 14,
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

        // get and set uniforms
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

        let perspective_matrix_location =
            program.get_uniform_location(c"perspectiveMatrix").unwrap();

        program.set_used();
        program.set_uniform(perspective_matrix_location, matrix);
        program.set_unused();

        Self {
            gl,
            program,
            vertex_buffer_object,
            vertex_buffer,
            index_buffer,
            window,
            offset_location,
            perspective_matrix_location,
            perspective_matrix: matrix,
        }
    }

    fn display(&mut self) {
        self.gl.clear_color(0.5, 0.5, 0.5, 0.0);
        self.gl.clear(gl::COLOR_BUFFER_BIT);

        self.program.set_used();

        self.vertex_buffer_object.bind();
        self.program
            .set_uniform(self.offset_location, (0.0, 0.0, 0.0));
        self.gl.draw_elements(
            DrawMode::Triangles,
            INDEX_DATA.len() as GLsizei,
            IndexSize::UnsignedInt,
            0,
        );

        self.program
            .set_uniform(self.offset_location, (0.0, 0.0, -1.0));
        self.gl.draw_elements_base_vertex(
            DrawMode::Triangles,
            INDEX_DATA.len() as GLsizei,
            IndexSize::UnsignedInt,
            0,
            (NUMBER_OF_VERTICES / 2) as i32,
        );

        self.vertex_buffer_object.unbind();
        self.program.set_unused();
    }

    fn keyboard(&mut self, _key: Key, _action: Action, _modifier: Modifiers) {}

    fn reshape(&mut self, width: i32, height: i32) {
        let frustum_scale = 1.0;

        self.perspective_matrix[0] = frustum_scale / (width as f32 / height as f32);
        self.perspective_matrix[5] = frustum_scale;

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
