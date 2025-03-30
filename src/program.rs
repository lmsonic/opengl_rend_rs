use crate::GLHandle;

#[derive(Default)]
pub struct Program {
    id: GLHandle,
}

impl Program {
    pub fn set_used(&mut self) {
        unsafe { gl::UseProgram(self.id) };
    }
    pub fn set_unused(&mut self) {
        unsafe { gl::UseProgram(0) };
    }
}
