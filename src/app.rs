use glfw::{fail_on_errors, Action, Context, Key, Modifiers, PWindow};

pub trait Application {
    fn new(window: PWindow) -> Self;
    fn display(&mut self) {}
    fn keyboard(&mut self, _key: Key, _action: Action, _modifier: Modifiers) {}
    fn reshape(&mut self, _width: i32, _height: i32) {}
    fn window(&self) -> &PWindow;
    fn window_mut(&mut self) -> &mut PWindow;
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
pub fn run_app<A: Application>() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));

    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw
        .create_window(600, 600, "OpenGl", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    // Make the window's context current
    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    let mut app = A::new(window);

    // Loop until the user closes the window
    while !app.window().should_close() {
        // process events
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    app.window_mut().set_should_close(true)
                }
                glfw::WindowEvent::Key(key, _, action, modifier) => {
                    app.keyboard(key, action, modifier)
                }

                glfw::WindowEvent::FramebufferSize(width, height) => app.reshape(width, height),
                _ => {}
            }
        }

        // render
        app.display();

        // Swap front and back buffers
        app.window_mut().swap_buffers();

        // Poll for and process events
        glfw.poll_events();
    }
}
