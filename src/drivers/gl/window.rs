extern crate gl;
extern crate glfw;

use gl::{Clear, ClearColor, COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT, DEPTH_TEST};
use glfw::{fail_on_errors, Action, Context, Key};
use nalgebra::{Matrix4, Perspective3, Point3, Vector3};

use super::{ render_object::RenderObject, shader::Shader, viewport::Viewport};

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

  unsafe {
    gl::Enable(DEPTH_TEST);
  }

  // Cube
  let vertices: [f32; 24] = [
    // Position         // Description
    -1.0, -1.0, -1.0, // Back-bottom-left 0
     1.0, -1.0, -1.0, // Back-bottom-right 1
     1.0,  1.0, -1.0, // Back-top-right 2
    -1.0,  1.0, -1.0, // Back-top-left 3
    -1.0, -1.0,  1.0, // Front-bottom-left 4
     1.0, -1.0,  1.0, // Front-bottom-right 5
     1.0,  1.0,  1.0, // Front-top-right 6
    -1.0,  1.0,  1.0, // Front-top-left 7
];

let indices: [u32; 36] = [
    // Back face
    0, 1, 2, 2, 3, 0,
    // Right face
    1, 5, 6, 6, 2, 1,
    // Front face
    5, 4, 7, 7, 6, 5,
    // Left face
    4, 0, 3, 3, 7, 4,
    // Bottom face
    4, 5, 1, 1, 0, 4,
    // Top face
    3, 2, 6, 6, 7, 3,
];

  let vertex_source = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;

    uniform mat4 model;
    uniform mat4 view;
    uniform mat4 projection;

    void main() {
      gl_Position = projection * view * model * vec4(aPos, 1.0);
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
  let mut cube = RenderObject::new(&vertices, &indices);

  let viewport = Viewport::new(
    Point3::new(0.0, 0.0, 3.0),
    Point3::origin(),
    Vector3::y()
  );

  let aspect_ratio = 800.0 / 600.0;
  let projection = Perspective3::new(
    aspect_ratio, 
    nalgebra::convert(45.0f32.to_radians()), 
    0.1, 
    100.0).to_homogeneous();

  let mut angle: f32 = 0.0;

  while !window.should_close() {
    
    glfw.poll_events();
    for (_, event) in glfw::flush_messages(&events) {
      match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
          window.set_should_close(true)
        }
        _ => {}
      }
    }

    unsafe {
      ClearColor(0.2, 0.2, 0.2, 1.0);
      Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
    }

    angle += 0.0001;
    let rotation_matrix = Matrix4::<f32>::from_axis_angle(&Vector3::y_axis(), nalgebra::convert(angle));
    let view = viewport.get_view_matrix();
    //let model = Matrix4::<f32>::identity();
    let model = rotation_matrix;

    cube.render(&shader, vertices.len(), &model, &view, &projection);

    window.swap_buffers();
    process_input(&mut window);
  }
}

fn process_input(window: &mut glfw::Window) {
  if window.get_key(Key::Escape) == Action::Press {
    window.set_should_close(true);
  }
}