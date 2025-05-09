#![forbid(unsafe_code)]
use std::ffi::CString;

use gl::types::GLsizei;
use glam::{Mat4, Vec3};
use glfw::{Action, Key, Modifiers, PWindow};
use opengl_rend::app::{run_app, Application};
use opengl_rend::mesh::Mesh;
use opengl_rend::opengl::{Capability, ClearFlags, CullMode, DepthFunc, FrontFace};
use opengl_rend::program::{GLLocation, Shader, ShaderType};
use opengl_rend::{opengl::OpenGl, program::Program};

struct App {
    window: PWindow,
    gl: OpenGl,
    program: Program,
    camera_to_clip_uniform: GLLocation,
    model_to_camera_uniform: GLLocation,
    plane_mesh: Mesh,
    large_gimbal: Mesh,
    medium_gimbal: Mesh,
    small_gimbal: Mesh,
    ship_mesh: Mesh,
}

impl Application for App {
    fn new(mut window: PWindow) -> Self {
        let mut gl = OpenGl::new(&mut window);

        // initialize programs
        let vertex = CString::new(include_str!("pos_color_local_transform.vert")).unwrap();
        let fragment = CString::new(include_str!("color_mult_uniform.frag")).unwrap();
        let mut program = Program::new(&[
            Shader::new(&vertex, ShaderType::Vertex).unwrap(),
            Shader::new(&fragment, ShaderType::Fragment).unwrap(),
        ])
        .unwrap();

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

        let large_gimbal = Mesh::new("examples/oriented/meshes/LargeGimbal.xml").unwrap();
        let medium_gimbal = Mesh::new("examples/oriented/meshes/MediumGimbal.xml").unwrap();
        let small_gimbal = Mesh::new("examples/oriented/meshes/SmallGimbal.xml").unwrap();
        let ship_mesh = Mesh::new("examples/oriented/meshes/Ship.xml").unwrap();
        let plane_mesh = Mesh::new("examples/oriented/meshes/UnitPlane.xml").unwrap();

        let camera_to_clip_uniform = program.get_uniform_location(c"cameraToClip").unwrap();
        let model_to_camera_uniform = program.get_uniform_location(c"modelToCamera").unwrap();

        Self {
            window,
            gl,
            program,
            camera_to_clip_uniform,
            model_to_camera_uniform,
            plane_mesh,
            large_gimbal,
            medium_gimbal,
            small_gimbal,
            ship_mesh,
        }
    }

    fn display(&mut self) {
        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.gl.clear_depth(1.0);
        self.gl.clear(ClearFlags::Color | ClearFlags::Depth);

        // Draw
    }

    fn keyboard(&mut self, key: Key, action: Action, modifier: Modifiers) {
        if action == Action::Press || action == Action::Repeat {}
    }

    fn reshape(&mut self, width: i32, height: i32) {
        const FOV: f32 = 20.0;
        const Z_NEAR: f32 = 1.0;
        const Z_FAR: f32 = 500.0;
        let matrix = Mat4::perspective_rh_gl(
            f32::to_radians(FOV),
            width as f32 / height as f32,
            Z_NEAR,
            Z_FAR,
        );
        // self.program.set_uniform(location, value);

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
