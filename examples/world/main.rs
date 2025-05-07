#![forbid(unsafe_code)]
use std::ffi::CString;

use gl::types::GLsizei;
use glam::{Mat4, Vec3, Vec4};
use glfw::{Action, Key, Modifiers, PWindow};
use opengl_rend::app::{run_app, Application};
use opengl_rend::buffer::{Buffer, Target, Usage};
use opengl_rend::matrix_stack::{MatrixStack, PushStack};
use opengl_rend::mesh::Mesh;
use opengl_rend::opengl::{Capability, ClearFlags, CullMode, DepthFunc, FrontFace};
use opengl_rend::program::{GLBlockIndex, GLLocation, Shader, ShaderType};
use opengl_rend::{opengl::OpenGl, program::Program};

struct ProgramData {
    program: Program,
    model_to_world_matrix_uniform: GLLocation,
    global_matrix_uniform: GLBlockIndex,
    base_color_uniform: GLLocation,
}

const GLOBAL_MATRICES_BINDING_INDEX: u32 = 0;

fn load_program(vert: &str, frag: &str) -> ProgramData {
    let vert = CString::new(vert).unwrap();
    let frag = CString::new(frag).unwrap();
    let vert_shader = Shader::new(&vert, ShaderType::Vertex).unwrap();
    let frag_shader = Shader::new(&frag, ShaderType::Fragment).unwrap();
    let mut program = Program::new(&[vert_shader, frag_shader]).unwrap();

    let global_matrix_uniform = program.get_uniform_block_index(c"GlobalMatrices").unwrap();
    program.uniform_block_binding(global_matrix_uniform, GLOBAL_MATRICES_BINDING_INDEX);
    ProgramData {
        model_to_world_matrix_uniform: program.get_uniform_location(c"modelToWorld").unwrap(),
        global_matrix_uniform,
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
    global_matrices_buffer: Buffer<Mat4>,
}

const PARTHENON_COLUMN_HEIGHT: f32 = 5.0;
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 1000.0;
const FOV: f32 = 100.0;
impl App {
    #[allow(clippy::too_many_lines)]
    fn draw_parthenon(&mut self, stack: &mut MatrixStack) {
        const PARTHENON_WIDTH: f32 = 14.0;
        const PARTHENON_LENGTH: f32 = 20.0;
        const PARTHENON_BASE_HEIGHT: f32 = 1.0;
        const PARTHENON_TOP_HEIGHT: f32 = 2.0;
        const FRONT_Z: f32 = PARTHENON_LENGTH * 0.5 - 1.0;
        const RIGHT_X: f32 = PARTHENON_WIDTH * 0.5 - 1.0;
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

        for i in 0..(PARTHENON_WIDTH / 2.0) as usize {
            {
                let push = PushStack::new(stack);
                push.stack.translate(Vec3::new(
                    2.0f32.mul_add(i as f32, -(PARTHENON_WIDTH / 2.0)) + 1.0,
                    PARTHENON_BASE_HEIGHT,
                    FRONT_Z,
                ));
                self.draw_column(push.stack, PARTHENON_COLUMN_HEIGHT);
            }
            {
                let push = PushStack::new(stack);
                push.stack.translate(Vec3::new(
                    2.0f32.mul_add(i as f32, -(PARTHENON_WIDTH / 2.0)) + 1.0,
                    PARTHENON_BASE_HEIGHT,
                    -FRONT_Z,
                ));
                self.draw_column(push.stack, PARTHENON_COLUMN_HEIGHT);
            }
        }
        for i in 1..((PARTHENON_LENGTH - 2.0) / 2.0) as usize {
            {
                let push = PushStack::new(stack);
                push.stack.translate(Vec3::new(
                    RIGHT_X,
                    PARTHENON_BASE_HEIGHT,
                    2.0f32.mul_add(i as f32, -(PARTHENON_LENGTH / 2.0)) + 1.0,
                ));
                self.draw_column(push.stack, PARTHENON_COLUMN_HEIGHT);
            }
            {
                let push = PushStack::new(stack);
                push.stack.translate(Vec3::new(
                    -RIGHT_X,
                    PARTHENON_BASE_HEIGHT,
                    2.0f32.mul_add(i as f32, -(PARTHENON_LENGTH / 2.0)) + 1.0,
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
                PARTHENON_TOP_HEIGHT.mul_add(0.5, PARTHENON_COLUMN_HEIGHT + PARTHENON_BASE_HEIGHT),
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
            push.stack.scale(Vec3::new(
                0.8,
                COLUMN_BASE_HEIGHT.mul_add(-2.0, height),
                0.8,
            ));
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
            push.stack.translate(Vec3::new(0.0, trunk_height, 0.0));
            push.stack.scale(Vec3::new(3.0, cone_height, 3.0));

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

    fn calculate_camera_pos(&self) -> Vec3 {
        let phi = self.camera_spherical_coords.x.to_radians();
        let theta = (self.camera_spherical_coords.y + 90.0).to_radians();

        let (sin_phi, cos_phi) = phi.sin_cos();
        let (sin_theta, cos_theta) = theta.sin_cos();
        Vec3::new(sin_theta * cos_phi, cos_theta, sin_theta * sin_phi)
            * self.camera_spherical_coords.z
            + self.camera_target
    }
}

impl Application for App {
    fn new(mut window: PWindow) -> Self {
        let mut gl = OpenGl::new(&mut window);

        // initialize programs
        let uniform_color = load_program(
            include_str!("only_pos_world_transformUBO.vert"),
            include_str!("base_color.frag"),
        );
        let object_color = load_program(
            include_str!("pos_color_world_transformUBO.vert"),
            include_str!("passthrough_color.frag"),
        );
        let object_color_tint = load_program(
            include_str!("pos_color_world_transformUBO.vert"),
            include_str!("base_vertex_color.frag"),
        );

        let mut global_matrices_buffer = Buffer::new(Target::UniformBuffer);
        global_matrices_buffer.bind();
        global_matrices_buffer.reserve_data(2, Usage::StaticDraw);
        global_matrices_buffer.unbind();
        global_matrices_buffer.bind_range_bytes(
            GLOBAL_MATRICES_BINDING_INDEX,
            0,
            2 * std::mem::size_of::<Mat4>() as isize,
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
            global_matrices_buffer,
        }
    }

    fn display(&mut self) {
        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.gl.clear_depth(1.0);
        self.gl.clear(ClearFlags::Color | ClearFlags::Depth);

        // Draw
        let camera_position = self.calculate_camera_pos();
        let look_at = Mat4::look_at_rh(camera_position, self.camera_target, Vec3::Y);
        self.global_matrices_buffer.bind();
        self.global_matrices_buffer.update_data(&[look_at], 1);
        self.global_matrices_buffer.unbind();

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
        self.draw_forest(&mut model_matrix);
        {
            // Draw the building
            let push = PushStack::new(&mut model_matrix);
            push.stack.translate(Vec3::new(20.0, 0.0, -10.0));
            self.draw_parthenon(push.stack);
        }
        if self.look_at_point {
            self.gl.disable(Capability::DepthTest);

            let push = PushStack::new(&mut model_matrix);
            push.stack.translate(self.camera_target);
            push.stack.scale(Vec3::ONE);

            let p = &mut self.object_color;
            p.program.set_used();
            p.program
                .set_uniform(p.model_to_world_matrix_uniform, push.stack.top());

            self.cube_color_mesh.render(&mut self.gl);
            p.program.set_unused();

            self.gl.enable(Capability::DepthTest);
        }
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
            self.camera_spherical_coords.z = self.camera_spherical_coords.z.clamp(-5.0, 5.0);
            self.camera_target.y = self.camera_target.y.max(0.0);
            let position = self.calculate_camera_pos();
            println!("Target {}", self.camera_target);
            println!("Absolute Position {position}");
            println!("Distance {}", self.camera_target.distance(position));
            println!("Spherical coords {}", self.camera_spherical_coords);
        }
    }

    fn reshape(&mut self, width: i32, height: i32) {
        let matrix = Mat4::perspective_rh_gl(
            f32::to_radians(FOV),
            width as f32 / height as f32,
            Z_NEAR,
            Z_FAR,
        );

        self.global_matrices_buffer.bind();
        self.global_matrices_buffer.update_data(&[matrix], 0);
        self.global_matrices_buffer.unbind();

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
