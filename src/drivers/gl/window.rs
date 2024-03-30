extern crate gl;
extern crate glfw;

use glfw::{fail_on_errors, Action, Context, Key};

pub fn run() {

  let mut glfw = glfw::init(fail_on_errors!()).unwrap();
  glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
  glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
  glfw.window_hint(glfw::WindowHint::Resizable(false));

  let (mut window, events) = glfw.create_window(800, 600, "OpenGL Renderer", glfw::WindowMode::Windowed)
    .expect("Failed to create GLFW window.");

  window.make_current();
  window.set_key_polling(true);

  gl::load_with(|s| window.get_proc_address(s) as *const _);

  while !window.should_close() {
    process_input(&mut window);
    window.swap_buffers();
    glfw.poll_events();
  }
}

fn process_input(window: &mut glfw::Window) {
  if window.get_key(Key::Escape) == Action::Press {
    window.set_should_close(true);
  }
}