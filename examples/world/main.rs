#![forbid(unsafe_code)]
use std::ffi::CString;

use gl::types::GLsizei;
use glam::{Mat4, Vec3};
use glfw::{Action, Key, Modifiers, PWindow};
use opengl_rend::app::{run_app, Application};
use opengl_rend::matrix_stack::MatrixStack;
use opengl_rend::opengl::{Capability, ClearFlags, CullMode, DepthFunc, FrontFace};
use opengl_rend::program::{GLLocation, Shader, ShaderType};
use opengl_rend::{opengl::OpenGl, program::Program};

struct ProgramData {
    program: Program,
    model_to_world_matrix_uniform: GLLocation,
    world_to_camera_matrix_uniform: GLLocation,
    camera_to_clip_matrix_uniform: GLLocation,
    base_color_uniform: GLLocation,
}

fn load_program(vert: &str, frag: &str) -> ProgramData {
    let vert = CString::new(vert).unwrap();
    let frag = CString::new(frag).unwrap();
    let vert_shader = Shader::new(&vert, ShaderType::Vertex).unwrap();
    let frag_shader = Shader::new(&frag, ShaderType::Fragment).unwrap();
    let mut program = Program::new(&[vert_shader, frag_shader]).unwrap();

    ProgramData {
        model_to_world_matrix_uniform: program.get_uniform_location(c"modelToWorld").unwrap(),
        world_to_camera_matrix_uniform: program.get_uniform_location(c"worldToCamera").unwrap(),
        camera_to_clip_matrix_uniform: program.get_uniform_location(c"cameraToClip").unwrap(),
        base_color_uniform: program.get_uniform_location(c"baseColor").unwrap_or(-1),
        program,
    }
}

struct App {
    window: PWindow,
    gl: OpenGl,
    uniform_color: ProgramData,
    object_color: ProgramData,
    object_color_tint: ProgramData,
    matrix_stack: MatrixStack,
    camera_target: Vec3,
    camera_spherical_coords: Vec3,
}

fn set_camera_matrix(data: &mut ProgramData, matrix: Mat4) {
    data.program.set_used();
    data.program
        .set_uniform(data.camera_to_clip_matrix_uniform, matrix);
    data.program.set_unused();
}

impl App {
    fn set_camera_matrices(&mut self, matrix: Mat4) {
        set_camera_matrix(&mut self.uniform_color, matrix);
        set_camera_matrix(&mut self.object_color, matrix);
        set_camera_matrix(&mut self.object_color_tint, matrix);
    }

    fn calculate_camera_pos(&self) -> Vec3 {
        let phi = self.camera_spherical_coords.x.to_radians();
        let theta = (self.camera_spherical_coords.y + 90.0).to_radians();

        let (sinp, cosp) = phi.sin_cos();
        let (sint, cost) = theta.sin_cos();
        Vec3::new(sint * cosp, cost, sint * sinp) * self.camera_spherical_coords.z
            + self.camera_target
    }
}

impl Application for App {
    fn new(mut window: PWindow) -> App {
        let mut gl = OpenGl::new(&mut window);

        // initialize programs
        let uniform_color = load_program(
            include_str!("only_pos_world_transform.vert"),
            include_str!("base_color.frag"),
        );
        let object_color = load_program(
            include_str!("pos_color_world_transform.vert"),
            include_str!("passthrough_color.frag"),
        );
        let object_color_tint = load_program(
            include_str!("pos_color_world_transform.vert"),
            include_str!("base_vertex_color.frag"),
        );

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

        Self {
            gl,
            window,
            uniform_color,
            object_color,
            object_color_tint,
            camera_target: Vec3::new(0.0, 0.4, 0.0),
            matrix_stack: MatrixStack::new(),
            camera_spherical_coords: Vec3::new(67.5, -46.0, 150.0),
        }
    }

    fn display(&mut self) {
        self.gl.clear_color(0.1, 0.1, 0.1, 0.0);
        self.gl.clear_depth(1.0);
        self.gl.clear(ClearFlags::Color | ClearFlags::Depth);

        // Draw

        let camera_position = self.calculate_camera_pos();
        let look_at = Mat4::look_at_rh(camera_position, self.camera_target, Vec3::Y);
        self.set_camera_matrices(look_at);
    }

    fn keyboard(&mut self, key: Key, action: Action, _modifier: Modifiers) {
        if action == Action::Press || action == Action::Repeat {
            match key {
                _ => {}
            }
        }
    }

    fn reshape(&mut self, width: i32, height: i32) {
        const Z_NEAR: f32 = 0.1;
        const Z_FAR: f32 = 100.0;
        let matrix = Mat4::perspective_rh_gl(
            f32::to_radians(45.0),
            width as f32 / height as f32,
            Z_NEAR,
            Z_FAR,
        );

        self.set_camera_matrices(matrix);

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
