#![forbid(unsafe_code)]
use std::ffi::CString;

use gl::types::GLsizei;
use glam::{Mat4, Vec3, Vec4};
use glfw::{Action, Key, Modifiers, PWindow};
use opengl_rend::app::{run_app, Application};
use opengl_rend::matrix_stack::{MatrixStack, PushStack};
use opengl_rend::mesh::Mesh;
use opengl_rend::opengl::{Capability, ClearFlags, CullMode, DepthFunc, FrontFace, PolygonMode};
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

const FOREST: [[f32; 4]; 98] = [
    [-45.0, -40.0, 2.0, 3.0],
    [-42.0, -35.0, 2.0, 3.0],
    [-39.0, -29.0, 2.0, 4.0],
    [-44.0, -26.0, 3.0, 3.0],
    [-40.0, -22.0, 2.0, 4.0],
    [-36.0, -15.0, 3.0, 3.0],
    [-41.0, -11.0, 2.0, 3.0],
    [-37.0, -6.0, 3.0, 3.0],
    [-45.0, 0.0, 2.0, 3.0],
    [-39.0, 4.0, 3.0, 4.0],
    [-36.0, 8.0, 2.0, 3.0],
    [-44.0, 13.0, 3.0, 3.0],
    [-42.0, 17.0, 2.0, 3.0],
    [-38.0, 23.0, 3.0, 4.0],
    [-41.0, 27.0, 2.0, 3.0],
    [-39.0, 32.0, 3.0, 3.0],
    [-44.0, 37.0, 3.0, 4.0],
    [-36.0, 42.0, 2.0, 3.0],
    [-32.0, -45.0, 2.0, 3.0],
    [-30.0, -42.0, 2.0, 4.0],
    [-34.0, -38.0, 3.0, 5.0],
    [-33.0, -35.0, 3.0, 4.0],
    [-29.0, -28.0, 2.0, 3.0],
    [-26.0, -25.0, 3.0, 5.0],
    [-35.0, -21.0, 3.0, 4.0],
    [-31.0, -17.0, 3.0, 3.0],
    [-28.0, -12.0, 2.0, 4.0],
    [-29.0, -7.0, 3.0, 3.0],
    [-26.0, -1.0, 2.0, 4.0],
    [-32.0, 6.0, 2.0, 3.0],
    [-30.0, 10.0, 3.0, 5.0],
    [-33.0, 14.0, 2.0, 4.0],
    [-35.0, 19.0, 3.0, 4.0],
    [-28.0, 22.0, 2.0, 3.0],
    [-33.0, 26.0, 3.0, 3.0],
    [-29.0, 31.0, 3.0, 4.0],
    [-32.0, 38.0, 2.0, 3.0],
    [-27.0, 41.0, 3.0, 4.0],
    [-31.0, 45.0, 2.0, 4.0],
    [-28.0, 48.0, 3.0, 5.0],
    [-25.0, -48.0, 2.0, 3.0],
    [-20.0, -42.0, 3.0, 4.0],
    [-22.0, -39.0, 2.0, 3.0],
    [-19.0, -34.0, 2.0, 3.0],
    [-23.0, -30.0, 3.0, 4.0],
    [-24.0, -24.0, 2.0, 3.0],
    [-16.0, -21.0, 2.0, 3.0],
    [-17.0, -17.0, 3.0, 3.0],
    [-25.0, -13.0, 2.0, 4.0],
    [-23.0, -8.0, 2.0, 3.0],
    [-17.0, -2.0, 3.0, 3.0],
    [-16.0, 1.0, 2.0, 3.0],
    [-19.0, 4.0, 3.0, 3.0],
    [-22.0, 8.0, 2.0, 4.0],
    [-21.0, 14.0, 2.0, 3.0],
    [-16.0, 19.0, 2.0, 3.0],
    [-23.0, 24.0, 3.0, 3.0],
    [-18.0, 28.0, 2.0, 4.0],
    [-24.0, 31.0, 2.0, 3.0],
    [-20.0, 36.0, 2.0, 3.0],
    [-22.0, 41.0, 3.0, 3.0],
    [-21.0, 45.0, 2.0, 3.0],
    [-12.0, -40.0, 2.0, 4.0],
    [-11.0, -35.0, 3.0, 3.0],
    [-10.0, -29.0, 1.0, 3.0],
    [-9.0, -26.0, 2.0, 2.0],
    [-6.0, -22.0, 2.0, 3.0],
    [-15.0, -15.0, 1.0, 3.0],
    [-8.0, -11.0, 2.0, 3.0],
    [-14.0, -6.0, 2.0, 4.0],
    [-12.0, 0.0, 2.0, 3.0],
    [-7.0, 4.0, 2.0, 2.0],
    [-13.0, 8.0, 2.0, 2.0],
    [-9.0, 13.0, 1.0, 3.0],
    [-13.0, 17.0, 3.0, 4.0],
    [-6.0, 23.0, 2.0, 3.0],
    [-12.0, 27.0, 1.0, 2.0],
    [-8.0, 32.0, 2.0, 3.0],
    [-10.0, 37.0, 3.0, 3.0],
    [-11.0, 42.0, 2.0, 2.0],
    [15.0, 5.0, 2.0, 3.0],
    [15.0, 10.0, 2.0, 3.0],
    [15.0, 15.0, 2.0, 3.0],
    [15.0, 20.0, 2.0, 3.0],
    [15.0, 25.0, 2.0, 3.0],
    [15.0, 30.0, 2.0, 3.0],
    [15.0, 35.0, 2.0, 3.0],
    [15.0, 40.0, 2.0, 3.0],
    [15.0, 45.0, 2.0, 3.0],
    [25.0, 5.0, 2.0, 3.0],
    [25.0, 10.0, 2.0, 3.0],
    [25.0, 15.0, 2.0, 3.0],
    [25.0, 20.0, 2.0, 3.0],
    [25.0, 25.0, 2.0, 3.0],
    [25.0, 30.0, 2.0, 3.0],
    [25.0, 35.0, 2.0, 3.0],
    [25.0, 40.0, 2.0, 3.0],
    [25.0, 45.0, 2.0, 3.0],
];

struct App {
    window: PWindow,
    gl: OpenGl,
    uniform_color: ProgramData,
    object_color: ProgramData,
    uniform_color_tint: ProgramData,
    camera_target: Vec3,
    camera_spherical_coords: Vec3,
    plane_mesh: Mesh,
    cone_mesh: Mesh,
    cube_color_mesh: Mesh,
    cube_tint_mesh: Mesh,
    cylinder_mesh: Mesh,
    look_at_point: bool,
}

fn set_camera_matrix(data: &mut ProgramData, matrix: Mat4) {
    data.program.set_used();
    data.program
        .set_uniform(data.camera_to_clip_matrix_uniform, matrix);
    data.program.set_unused();
}

const PARTHENON_COLUMN_HEIGHT: f32 = 5.0;
impl App {
    fn draw_parthenon(&mut self, stack: &mut MatrixStack) {
        const PARTHENON_WIDTH: f32 = 14.0;
        const PARTHENON_LENGTH: f32 = 20.0;
        const PARTHENON_BASE_HEIGHT: f32 = 1.0;
        const PARTHENON_TOP_HEIGHT: f32 = 2.0;
        {
            // draw base
            let push = PushStack::new(stack);
            push.stack.scale(Vec3::new(
                PARTHENON_WIDTH,
                PARTHENON_BASE_HEIGHT,
                PARTHENON_LENGTH,
            ));
            push.stack.translate(Vec3::new(0.0, 0.5, 0.0));

            let p = &mut self.uniform_color_tint;

            p.program.set_used();
            p.program
                .set_uniform(p.model_to_world_matrix_uniform, push.stack.top());
            p.program
                .set_uniform(p.base_color_uniform, (0.9, 0.9, 0.9, 0.9));
            self.cube_tint_mesh.render(&mut self.gl);
            p.program.set_unused();
        }
        {
            // draw top
            let push = PushStack::new(stack);
            push.stack.translate(Vec3::new(
                0.0,
                PARTHENON_COLUMN_HEIGHT + PARTHENON_BASE_HEIGHT,
                0.0,
            ));

            push.stack.scale(Vec3::new(
                PARTHENON_WIDTH,
                PARTHENON_TOP_HEIGHT,
                PARTHENON_LENGTH,
            ));
            push.stack.translate(Vec3::new(0.0, 0.5, 0.0));

            let p = &mut self.uniform_color_tint;

            p.program.set_used();
            p.program
                .set_uniform(p.model_to_world_matrix_uniform, push.stack.top());
            p.program
                .set_uniform(p.base_color_uniform, (0.9, 0.9, 0.9, 0.9));
            self.cube_tint_mesh.render(&mut self.gl);
            p.program.set_unused();
        }

        const FRONT_Z: f32 = PARTHENON_LENGTH * 0.5 - 1.0;
        const RIGHT_X: f32 = PARTHENON_WIDTH * 0.5 - 1.0;
        for i in 0..=(PARTHENON_WIDTH / 2.0) as usize {
            {
                let push = PushStack::new(stack);
                push.stack.translate(Vec3::new(
                    (2.0 * i as f32) - (PARTHENON_WIDTH / 2.0) + 1.0,
                    PARTHENON_BASE_HEIGHT,
                    FRONT_Z,
                ));
                self.draw_column(push.stack, PARTHENON_COLUMN_HEIGHT);
            }
            {
                let push = PushStack::new(stack);
                push.stack.translate(Vec3::new(
                    (2.0 * i as f32) - (PARTHENON_WIDTH / 2.0) + 1.0,
                    PARTHENON_BASE_HEIGHT,
                    -FRONT_Z,
                ));
                self.draw_column(push.stack, PARTHENON_COLUMN_HEIGHT);
            }
        }
        for i in 1..=((PARTHENON_LENGTH - 2.0) / 2.0) as usize {
            {
                let push = PushStack::new(stack);
                push.stack.translate(Vec3::new(
                    RIGHT_X,
                    PARTHENON_BASE_HEIGHT,
                    (2.0 * i as f32) - (PARTHENON_LENGTH / 2.0) + 1.0,
                ));
                self.draw_column(push.stack, PARTHENON_COLUMN_HEIGHT);
            }
            {
                let push = PushStack::new(stack);
                push.stack.translate(Vec3::new(
                    -RIGHT_X,
                    PARTHENON_BASE_HEIGHT,
                    (2.0 * i as f32) - (PARTHENON_LENGTH / 2.0) + 1.0,
                ));
                self.draw_column(push.stack, PARTHENON_COLUMN_HEIGHT);
            }
        }
        {
            // draw interior
            let push = PushStack::new(stack);
            push.stack.translate(Vec3::Y);
            push.stack.scale(Vec3::new(
                PARTHENON_WIDTH - 6.0,
                PARTHENON_COLUMN_HEIGHT,
                PARTHENON_LENGTH - 6.0,
            ));
            push.stack.translate(Vec3::new(0.0, 0.5, 0.0));

            let p = &mut self.object_color;
            p.program.set_used();
            p.program
                .set_uniform(p.model_to_world_matrix_uniform, push.stack.top());
            self.cube_color_mesh.render(&mut self.gl);
            p.program.set_unused();
        }
        {
            // draw headpiece
            let push = PushStack::new(stack);
            push.stack.translate(Vec3::new(
                0.0,
                PARTHENON_COLUMN_HEIGHT + PARTHENON_BASE_HEIGHT + PARTHENON_TOP_HEIGHT * 0.5,
                PARTHENON_LENGTH * 0.5,
            ));
            push.stack.rotate_x(-135.0);
            push.stack.rotate_y(45.0);

            let p = &mut self.object_color;
            p.program.set_used();
            p.program
                .set_uniform(p.model_to_world_matrix_uniform, push.stack.top());
            self.cube_color_mesh.render(&mut self.gl);
            p.program.set_unused();
        }
    }

    fn draw_column(&mut self, stack: &mut MatrixStack, height: f32) {
        const COLUMN_BASE_HEIGHT: f32 = 0.25;
        {
            // draw bottom
            let push = PushStack::new(stack);
            push.stack.scale(Vec3::new(1.0, COLUMN_BASE_HEIGHT, 1.0));
            push.stack.translate(Vec3::new(0.0, 0.5, 0.0));

            let p = &mut self.uniform_color_tint;
            p.program.set_used();
            p.program
                .set_uniform(p.model_to_world_matrix_uniform, push.stack.top());
            p.program.set_uniform(p.base_color_uniform, Vec4::ONE);
            self.cube_tint_mesh.render(&mut self.gl);
            p.program.set_unused();
        }
        {
            // draw top
            let push = PushStack::new(stack);
            push.stack
                .translate(Vec3::new(0.0, height - COLUMN_BASE_HEIGHT, 0.0));
            push.stack.scale(Vec3::new(1.0, COLUMN_BASE_HEIGHT, 1.0));
            push.stack.translate(Vec3::new(0.0, 0.5, 0.0));

            let p = &mut self.uniform_color_tint;
            p.program.set_used();
            p.program
                .set_uniform(p.model_to_world_matrix_uniform, push.stack.top());
            p.program
                .set_uniform(p.base_color_uniform, (0.9, 0.9, 0.9, 0.9));
            self.cube_tint_mesh.render(&mut self.gl);
            p.program.set_unused();
        }
        {
            // draw main column
            let push = PushStack::new(stack);
            push.stack
                .translate(Vec3::new(0.0, COLUMN_BASE_HEIGHT, 0.0));
            push.stack
                .scale(Vec3::new(0.8, height - COLUMN_BASE_HEIGHT * 2.0, 0.8));
            push.stack.translate(Vec3::new(0.0, 0.5, 0.0));

            let p = &mut self.uniform_color_tint;
            p.program.set_used();
            p.program
                .set_uniform(p.model_to_world_matrix_uniform, push.stack.top());
            p.program
                .set_uniform(p.base_color_uniform, (0.9, 0.9, 0.9, 0.9));
            self.cylinder_mesh.render(&mut self.gl);
            p.program.set_unused();
        }
    }

    fn draw_tree(&mut self, stack: &mut MatrixStack, trunk_height: f32, cone_height: f32) {
        {
            // draw trunk
            let push = PushStack::new(stack);
            push.stack.scale(Vec3::new(1.0, trunk_height, 1.0));
            push.stack.translate(Vec3::new(0.0, 0.5, 0.0));

            let p = &mut self.uniform_color_tint;

            p.program.set_used();
            p.program
                .set_uniform(p.model_to_world_matrix_uniform, push.stack.top());
            p.program
                .set_uniform(p.base_color_uniform, (0.694, 0.4, 0.106, 1.0));
            self.cylinder_mesh.render(&mut self.gl);
            p.program.set_unused();
        }
        {
            // draw treetop
            let push = PushStack::new(stack);
            push.stack.scale(Vec3::new(3.0, cone_height, 3.0));
            push.stack.translate(Vec3::new(0.0, trunk_height, 0.0));

            let p = &mut self.uniform_color_tint;

            p.program.set_used();
            p.program
                .set_uniform(p.model_to_world_matrix_uniform, push.stack.top());
            p.program
                .set_uniform(p.base_color_uniform, (0.0, 1.0, 0.0, 1.0));
            self.cone_mesh.render(&mut self.gl);
            p.program.set_unused();
        }
    }
    fn draw_forest(&mut self, model_matrix: &mut MatrixStack) {
        for [x_pos, z_pos, trunk_height, cone_height] in FOREST {
            let push = PushStack::new(model_matrix);
            push.stack.translate(Vec3::new(x_pos, 0.0, z_pos));
            self.draw_tree(push.stack, trunk_height, cone_height);
        }
    }
    fn set_camera_matrices(&mut self, matrix: Mat4) {
        set_camera_matrix(&mut self.uniform_color, matrix);
        set_camera_matrix(&mut self.object_color, matrix);
        set_camera_matrix(&mut self.uniform_color_tint, matrix);
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
        // gl.enable(Capability::CullFace);
        // gl.cull_face(CullMode::Back);
        // gl.front_face(FrontFace::CW);
        // gl.polygon_mode(PolygonMode::Line);

        // enable depth test
        gl.enable(Capability::DepthTest);
        gl.set_depth_mask(true);
        gl.depth_func(DepthFunc::LessEqual);
        gl.depth_range(0.0, 1.0);

        let cone_mesh = Mesh::new("examples/world/meshes/UnitConeTint.xml").unwrap();
        let cylinder_mesh = Mesh::new("examples/world/meshes/UnitCylinderTint.xml").unwrap();
        let cube_color_mesh = Mesh::new("examples/world/meshes/UnitCubeColor.xml").unwrap();
        let cube_tint_mesh = Mesh::new("examples/world/meshes/UnitCubeTint.xml").unwrap();
        let plane_mesh = Mesh::new("examples/world/meshes/UnitPlane.xml").unwrap();

        Self {
            gl,
            window,
            uniform_color,
            object_color,
            uniform_color_tint: object_color_tint,
            camera_target: Vec3::new(0.0, 0.4, 0.0),
            camera_spherical_coords: Vec3::new(67.5, -46.0, 150.0),
            plane_mesh,
            cone_mesh,
            cylinder_mesh,
            cube_tint_mesh,
            cube_color_mesh,
            look_at_point: false,
        }
    }

    fn display(&mut self) {
        self.gl.clear_color(0.2, 0.2, 0.2, 0.0);
        self.gl.clear_depth(1.0);
        self.gl.clear(ClearFlags::Color | ClearFlags::Depth);

        // Draw
        let camera_position = self.calculate_camera_pos();
        let look_at = Mat4::look_at_rh(camera_position, self.camera_target, Vec3::Y);
        self.set_camera_matrices(look_at);

        let mut model_matrix = MatrixStack::new();
        {
            // Draw ground
            let push = PushStack::new(&mut model_matrix);
            push.stack.scale(Vec3::new(1000.0, 1.0, 1000.0));
            let program_data = &mut self.uniform_color;
            program_data.program.set_used();
            program_data
                .program
                .set_uniform(program_data.model_to_world_matrix_uniform, push.stack.top());
            program_data
                .program
                .set_uniform(program_data.base_color_uniform, (0.302, 0.416, 0.0589, 1.0));
            self.plane_mesh.render(&mut self.gl);
            program_data.program.set_unused();
        }
        // self.draw_forest(&mut model_matrix);
        // {
        //     // Draw the building
        //     let push = PushStack::new(&mut model_matrix);
        //     push.stack.translate(Vec3::new(20.0, 0.0, -10.0));
        //     self.draw_parthenon(push.stack);
        // }
        // if self.look_at_point {
        //     self.gl.disable(Capability::DepthTest);
        //     let identity = Mat4::IDENTITY;
        //     let push = PushStack::new(&mut model_matrix);
        //     let camera_direction = self.camera_target - camera_position;
        //     push.stack
        //         .translate(Vec3::new(0.0, 0.0, -camera_direction.length()));
        //     push.stack.scale(Vec3::ONE);
        //     let p = &mut self.object_color;
        //     p.program.set_used();
        //     p.program
        //         .set_uniform(p.model_to_world_matrix_uniform, push.stack.top());
        //     p.program
        //         .set_uniform(p.world_to_camera_matrix_uniform, identity);
        //     self.cube_color_mesh.render(&mut self.gl);
        //     p.program.set_unused();
        //     self.gl.enable(Capability::DepthTest);
        // }
    }

    fn keyboard(&mut self, key: Key, action: Action, modifier: Modifiers) {
        let modifier = if modifier.contains(Modifiers::Shift) {
            0.1
        } else {
            1.0
        };
        if action == Action::Press || action == Action::Repeat {
            match key {
                Key::E => self.camera_target.y -= 4.0 * modifier,
                Key::Q => self.camera_target.y += 4.0 * modifier,
                Key::A => self.camera_target.x -= 4.0 * modifier,
                Key::D => self.camera_target.x += 4.0 * modifier,
                Key::W => self.camera_target.z -= 4.0 * modifier,
                Key::S => self.camera_target.z += 4.0 * modifier,

                Key::J => self.camera_spherical_coords.x -= 11.0 * modifier,
                Key::L => self.camera_spherical_coords.x += 11.0 * modifier,
                Key::I => self.camera_spherical_coords.y -= 11.0 * modifier,
                Key::K => self.camera_spherical_coords.y += 11.0 * modifier,
                Key::O => self.camera_spherical_coords.z -= 5.0 * modifier,
                Key::U => self.camera_spherical_coords.z += 5.0 * modifier,
                Key::Space => {
                    self.look_at_point = !self.look_at_point;
                    println!("look at point {}", self.look_at_point);
                    println!("Target {}", self.camera_target);
                }
                _ => {}
            }
            self.camera_spherical_coords.y = self.camera_spherical_coords.y.clamp(-78.75, -1.0);
            self.camera_spherical_coords.z = self.camera_spherical_coords.z.min(5.0);
            self.camera_target.y = self.camera_target.y.min(0.0);
            let position = self.calculate_camera_pos();
            println!("Target {}", self.camera_target);
            println!("Absolute Position {}", position);
            println!("Distance {}", self.camera_target.distance(position));
            println!("Spherical coords {}", self.camera_spherical_coords);
        }
    }

    fn reshape(&mut self, width: i32, height: i32) {
        const Z_NEAR: f32 = 0.1;
        const Z_FAR: f32 = 100.0;
        let matrix = Mat4::perspective_rh_gl(
            f32::to_radians(100.0),
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
