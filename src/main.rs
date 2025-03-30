mod buffer;
mod opengl;
mod program;
mod vertex_attributes;

use std::ptr;

use buffer::Buffer;
use opengl::OpenGl;
use program::Program;
use vertex_attributes::{Type, VertexAttribute};
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

pub trait Framework {
    fn new() -> App;
    fn display(&mut self) {}
    fn keyboard(&mut self, event: KeyEvent) {}
}

type GLHandle = gl::types::GLuint;

struct App {
    window: Option<Window>,
    gl: OpenGl,
    program: Program,
    vertex_buffer: Buffer<f32>,
}

const VERTEX_POSITIONS: [f32; 12] = [
    0.75, 0.75, 0.0, 1.0, 0.75, -0.75, 0.0, 1.0, -0.75, -0.75, 0.0, 1.0,
];

impl Framework for App {
    fn new() -> App {
        todo!()
    }

    fn display(&mut self) {
        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.gl.clear(gl::COLOR_BUFFER_BIT);

        self.program.set_used();

        self.vertex_buffer.bind();

        let vertex_attribute = VertexAttribute::new(0, 4, Type::Float, gl::FALSE, 0, ptr::null());
        vertex_attribute.enable();
        vertex_attribute.create();

        self.gl.draw_arrays(gl::TRIANGLES, 0, 3);
        vertex_attribute.disable();

        self.program.set_unused();
    }

    fn keyboard(&mut self, event: KeyEvent) {}
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let Some(window) = &self.window {
            if window.id() != id {
                return;
            }
        }
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.
                self.display();
                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.keyboard(event);
            }
            _ => (),
        }
    }
}

fn main() -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app)
}
