use gl::types::GLint;

mod private {
    pub trait Sealed {}
}
pub trait SetUniform: private::Sealed {
    fn set_uniform(&self, location: GLint);
}

impl private::Sealed for f32 {}

impl SetUniform for f32 {
    fn set_uniform(&self, location: GLint) {
        unsafe { gl::Uniform1f(location, *self) }
    }
}

impl private::Sealed for (f32, f32) {}

impl SetUniform for (f32, f32) {
    fn set_uniform(&self, location: GLint) {
        unsafe { gl::Uniform2f(location, self.0, self.1) }
    }
}

impl private::Sealed for (f32, f32, f32) {}

impl SetUniform for (f32, f32, f32) {
    fn set_uniform(&self, location: GLint) {
        unsafe { gl::Uniform3f(location, self.0, self.1, self.2) }
    }
}
impl private::Sealed for (f32, f32, f32, f32) {}

impl SetUniform for (f32, f32, f32, f32) {
    fn set_uniform(&self, location: GLint) {
        unsafe { gl::Uniform4f(location, self.0, self.1, self.2, self.3) }
    }
}

impl private::Sealed for i32 {}

impl SetUniform for i32 {
    fn set_uniform(&self, location: GLint) {
        unsafe { gl::Uniform1i(location, *self) }
    }
}

impl private::Sealed for (i32, i32) {}

impl SetUniform for (i32, i32) {
    fn set_uniform(&self, location: GLint) {
        unsafe { gl::Uniform2i(location, self.0, self.1) }
    }
}

impl private::Sealed for (i32, i32, i32) {}

impl SetUniform for (i32, i32, i32) {
    fn set_uniform(&self, location: GLint) {
        unsafe { gl::Uniform3i(location, self.0, self.1, self.2) }
    }
}
impl private::Sealed for (i32, i32, i32, i32) {}

impl SetUniform for (i32, i32, i32, i32) {
    fn set_uniform(&self, location: GLint) {
        unsafe { gl::Uniform4i(location, self.0, self.1, self.2, self.3) }
    }
}

impl private::Sealed for u32 {}

impl SetUniform for u32 {
    fn set_uniform(&self, location: GLint) {
        unsafe { gl::Uniform1ui(location, *self) }
    }
}

impl private::Sealed for (u32, u32) {}

impl SetUniform for (u32, u32) {
    fn set_uniform(&self, location: GLint) {
        unsafe { gl::Uniform2ui(location, self.0, self.1) }
    }
}

impl private::Sealed for (u32, u32, u32) {}

impl SetUniform for (u32, u32, u32) {
    fn set_uniform(&self, location: GLint) {
        unsafe { gl::Uniform3ui(location, self.0, self.1, self.2) }
    }
}
impl private::Sealed for (u32, u32, u32, u32) {}

impl SetUniform for (u32, u32, u32, u32) {
    fn set_uniform(&self, location: GLint) {
        unsafe { gl::Uniform4ui(location, self.0, self.1, self.2, self.3) }
    }
}
