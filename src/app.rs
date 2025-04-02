use glfw::{fail_on_errors, Action, Context, Key, Modifiers, Window};

pub trait Application {
    fn new(window: &mut Window) -> Self;
    fn display(&mut self) {}
    fn keyboard(&mut self, key: Key, action: Action, modifier: Modifiers) {}
    fn reshape(&mut self, width: i32, height: i32) {}
}

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

    let mut app = A::new(&mut window);

    // Loop until the user closes the window
    while !window.should_close() {
        // process events
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
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
        window.swap_buffers();

        // Poll for and process events
        glfw.poll_events();
    }
}
