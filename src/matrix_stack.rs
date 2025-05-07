use std::ops::Mul;

use glam::{Mat4, Quat, Vec3};

pub struct MatrixStack {
    stack: Vec<Mat4>,
    current_matrix: Mat4,
}

impl MatrixStack {
    #[must_use] pub const fn new() -> Self {
        Self {
            stack: vec![],
            current_matrix: Mat4::IDENTITY,
        }
    }
    #[must_use] pub const fn with_initial_matrix(mat: Mat4) -> Self {
        Self {
            stack: vec![],
            current_matrix: mat,
        }
    }
    pub fn push(&mut self) {
        self.stack.push(self.current_matrix);
    }
    pub fn pop(&mut self) {
        if let Some(value) = self.stack.pop() {
            self.current_matrix = value;
        }
    }
    pub fn reset(&mut self) {
        if let Some(value) = self.stack.last() {
            self.current_matrix = *value;
        }
    }
    #[must_use] pub const fn top(&self) -> Mat4 {
        self.current_matrix
    }
    pub fn rotate_rad(&mut self, axis: Vec3, angle_rad: f32) {
        let q = Quat::from_axis_angle(axis, angle_rad);
        self.current_matrix *= Mat4::from_quat(q);
    }
    pub fn rotate(&mut self, axis: Vec3, angle_deg: f32) {
        let q = Quat::from_axis_angle(axis, angle_deg.to_radians());
        self.current_matrix *= Mat4::from_quat(q);
    }
    pub fn rotate_x(&mut self, angle_deg: f32) {
        self.current_matrix *= Mat4::from_rotation_x(angle_deg.to_radians());
    }
    pub fn rotate_y(&mut self, angle_deg: f32) {
        self.current_matrix *= Mat4::from_rotation_y(angle_deg.to_radians());
    }
    pub fn rotate_z(&mut self, angle_deg: f32) {
        self.current_matrix *= Mat4::from_rotation_z(angle_deg.to_radians());
    }
    pub fn scale(&mut self, scale: Vec3) {
        self.current_matrix *= Mat4::from_scale(scale);
    }
    pub fn uniform_scale(&mut self, scale: f32) {
        self.current_matrix *= Mat4::from_scale(Vec3::ONE * scale);
    }
    pub fn translate(&mut self, translate: Vec3) {
        self.current_matrix *= Mat4::from_translation(translate);
    }

    pub fn look_at(&mut self, eye: Vec3, target: Vec3, up: Vec3) {
        self.current_matrix *= Mat4::look_at_rh(eye, target, up);
    }
    pub fn perspective(&mut self, fov: f32, aspect_ratio: f32, z_near: f32, z_far: f32) {
        self.current_matrix *= Mat4::perspective_rh_gl(fov, aspect_ratio, z_near, z_far);
    }
    pub fn orthographic(
        &mut self,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) {
        self.current_matrix *= Mat4::orthographic_rh_gl(left, right, bottom, top, near, far);
    }
    pub fn apply_matrix(&mut self, mat: Mat4) {
        self.current_matrix *= mat;
    }
    pub fn set_matrix(&mut self, mat: Mat4) {
        self.current_matrix = mat;
    }
    pub fn set_identity(&mut self) {
        self.current_matrix = Mat4::IDENTITY;
    }
}

pub struct PushStack<'a> {
    pub stack: &'a mut MatrixStack,
}

impl Drop for PushStack<'_> {
    fn drop(&mut self) {
        self.stack.pop();
    }
}

impl<'a> PushStack<'a> {
    pub fn new(stack: &'a mut MatrixStack) -> Self {
        stack.push();
        Self { stack }
    }
}

impl Mul<Mat4> for MatrixStack {
    type Output = ();

    fn mul(mut self, rhs: Mat4) -> Self::Output {
        self.current_matrix *= rhs;
    }
}

impl Default for MatrixStack {
    fn default() -> Self {
        Self::new()
    }
}
