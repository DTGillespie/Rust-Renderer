extern crate gl;
extern crate glfw;

use gl::{Clear, ClearColor, COLOR_BUFFER_BIT};
use glfw::{fail_on_errors, Action, Context, Key};

use super::{renderer::Renderer, shader::Shader};

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

  // Test Triangle 

  let vertices: [f32; 9] = [
    // Positions
    -0.5, -0.5, 0.0, // Vertex 1
     0.5, -0.5, 0.0, // Vertex 2
     0.0,  0.5, 0.0, // Vertex 3
  ];

  let vertex_source = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;

    void main() {
      gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
  "#;

  let fragment_source = r#"
    #version 330 core
    out vec4 FragColor;
    
    void main() {
      FragColor = vec4(1.0, 0.5, 0.2, 1.0); // orange color
    }
  "#;

  let shader = Shader::from_source(vertex_source, fragment_source);
  let mut renderer = Renderer::new(&vertices);

  while !window.should_close() {
    
    process_input(&mut window);

    unsafe {
      ClearColor(0.2, 0.2, 0.2, 1.0);
      Clear(COLOR_BUFFER_BIT);
    }

    renderer.render(&shader);

    window.swap_buffers();
    glfw.poll_events();
  }
}

fn process_input(window: &mut glfw::Window) {
  if window.get_key(Key::Escape) == Action::Press {
    window.set_should_close(true);
  }
}