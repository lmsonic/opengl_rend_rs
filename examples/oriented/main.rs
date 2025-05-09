#![forbid(unsafe_code)]
use std::ffi::CString;

use gl::types::GLsizei;
use glam::{Mat4, Vec3, Vec4};
use glfw::{Action, Key, Modifiers, PWindow};
use opengl_rend::app::{run_app, Application};
use opengl_rend::matrix_stack::{MatrixStack, PushStack};
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
    base_color_uniform: GLLocation,
    plane_mesh: Mesh,
    large_gimbal: Mesh,
    medium_gimbal: Mesh,
    small_gimbal: Mesh,
    ship_mesh: Mesh,
    gimbal_angles: Vec3,
    draw_gimbals: bool,
}

impl App {
    fn draw_gimbal(&mut self, stack: &mut MatrixStack, axis: GimbalAxis, color: Vec4) {
        if !self.draw_gimbals {
            return;
        }
        let push = PushStack::new(stack);
        match axis {
            GimbalAxis::X => {}
            GimbalAxis::Y => {
                push.stack.rotate_z(90.0);
                push.stack.rotate_x(90.0);
            }
            GimbalAxis::Z => {
                push.stack.rotate_y(90.0);
                push.stack.rotate_x(90.0);
            }
        }
        self.program.set_used();
        self.program.set_uniform(self.base_color_uniform, color);
        self.program
            .set_uniform(self.model_to_camera_uniform, push.stack.top());
        match axis {
            GimbalAxis::X => self.large_gimbal.render(&mut self.gl),
            GimbalAxis::Y => self.medium_gimbal.render(&mut self.gl),
            GimbalAxis::Z => self.small_gimbal.render(&mut self.gl),
        }
        self.program.set_unused();
    }
}

#[derive(Clone, Copy)]
enum GimbalAxis {
    X,
    Y,
    Z,
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
        let base_color_uniform = program.get_uniform_location(c"baseColor").unwrap();

        Self {
            window,
            gl,
            program,
            camera_to_clip_uniform,
            model_to_camera_uniform,
            base_color_uniform,
            plane_mesh,
            large_gimbal,
            medium_gimbal,
            small_gimbal,
            ship_mesh,
            gimbal_angles: Vec3::ZERO,
            draw_gimbals: true,
        }
    }

    fn display(&mut self) {
        self.gl.clear_color(0.1, 0.1, 0.1, 0.0);
        self.gl.clear_depth(1.0);
        self.gl.clear(ClearFlags::Color | ClearFlags::Depth);

        let mut stack = MatrixStack::new();
        stack.translate(Vec3::new(0.0, 0.0, -200.0));

        stack.rotate_x(self.gimbal_angles.x);
        self.draw_gimbal(&mut stack, GimbalAxis::X, Vec4::new(0.4, 0.4, 1.0, 1.0));
        stack.rotate_y(self.gimbal_angles.y);
        self.draw_gimbal(&mut stack, GimbalAxis::Y, Vec4::new(0.0, 1.0, 0.0, 1.0));
        stack.rotate_z(self.gimbal_angles.z);
        self.draw_gimbal(&mut stack, GimbalAxis::Z, Vec4::new(1.0, 0.3, 0.3, 1.0));

        stack.scale(Vec3::ONE * 3.0);
        stack.rotate_x(-90.0);

        self.program.set_used();
        self.program.set_uniform(self.base_color_uniform, Vec4::ONE);
        self.program
            .set_uniform(self.model_to_camera_uniform, stack.top());
        self.ship_mesh.render(&mut self.gl);
        self.program.set_unused();
    }

    fn keyboard(&mut self, key: Key, action: Action, _modifier: Modifiers) {
        const SMALL_ANGLE_INCREMENT: f32 = 9.0;
        if action == Action::Press || action == Action::Repeat {
            match key {
                Key::W => self.gimbal_angles.x += SMALL_ANGLE_INCREMENT,
                Key::S => self.gimbal_angles.x -= SMALL_ANGLE_INCREMENT,

                Key::A => self.gimbal_angles.y += SMALL_ANGLE_INCREMENT,
                Key::D => self.gimbal_angles.y -= SMALL_ANGLE_INCREMENT,

                Key::Q => self.gimbal_angles.z += SMALL_ANGLE_INCREMENT,
                Key::E => self.gimbal_angles.z -= SMALL_ANGLE_INCREMENT,
                Key::Space => self.draw_gimbals = !self.draw_gimbals,
                _ => {}
            }
        }
    }

    fn reshape(&mut self, width: i32, height: i32) {
        const FOV: f32 = 20.0;
        const Z_NEAR: f32 = 1.0;
        const Z_FAR: f32 = 1500.0;
        let matrix = Mat4::perspective_rh_gl(
            f32::to_radians(FOV),
            width as f32 / height as f32,
            Z_NEAR,
            Z_FAR,
        );
        self.program.set_used();
        self.program
            .set_uniform(self.camera_to_clip_uniform, matrix);
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
