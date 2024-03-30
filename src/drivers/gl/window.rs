extern crate gl;
extern crate glfw;

use gl::{Clear, ClearColor, COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT, DEPTH_TEST};
use glfw::{fail_on_errors, Action, Context, Key};
use nalgebra::{Matrix4, Perspective3, Point3, Vector3};

use super::{renderer::Renderer, shader::Shader, viewport::Viewport};

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

  let vertices: [f32; 216] = [
    // Front face
    -1.0, -1.0, 1.0, 0.0, 0.0, 1.0, // Bottom-left
     1.0, -1.0, 1.0, 0.0, 0.0, 1.0, // Bottom-right
     1.0,  1.0, 1.0, 0.0, 0.0, 1.0, // Top-right
     1.0,  1.0, 1.0, 0.0, 0.0, 1.0, // Top-right
    -1.0,  1.0, 1.0, 0.0, 0.0, 1.0, // Top-left
    -1.0, -1.0, 1.0, 0.0, 0.0, 1.0, // Bottom-left

    // Right face
     1.0, -1.0,  1.0, 0.0, 0.0, 1.0,  // Bottom-left
     1.0, -1.0, -1.0, 0.0, 0.0, 1.0,  // Bottom-right
     1.0,  1.0, -1.0, 0.0, 0.0, 1.0,  // Top-right
     1.0,  1.0, -1.0, 0.0, 0.0, 1.0,  // Top-right
     1.0,  1.0,  1.0, 0.0, 0.0, 1.0,  // Top-left
     1.0, -1.0,  1.0, 0.0, 0.0, 1.0,  // Bottom-left

    // Back face
     1.0, -1.0, -1.0, 0.0, 0.0, 1.0, // Bottom-left
    -1.0, -1.0, -1.0, 0.0, 0.0, 1.0, // Bottom-right
    -1.0,  1.0, -1.0, 0.0, 0.0, 1.0, // Top-right
    -1.0,  1.0, -1.0, 0.0, 0.0, 1.0, // Top-right
     1.0,  1.0, -1.0, 0.0, 0.0, 1.0, // Top-left
     1.0, -1.0, -1.0, 0.0, 0.0, 1.0, // Bottom-left

    // Left face
    -1.0, -1.0, -1.0, 0.0, 0.0, 1.0, // Bottom-left
    -1.0, -1.0,  1.0, 0.0, 0.0, 1.0, // Bottom-right
    -1.0,  1.0,  1.0, 0.0, 0.0, 1.0, // Top-right
    -1.0,  1.0,  1.0, 0.0, 0.0, 1.0, // Top-right
    -1.0,  1.0, -1.0, 0.0, 0.0, 1.0, // Top-left
    -1.0, -1.0, -1.0, 0.0, 0.0, 1.0, // Bottom-left

    // Bottom face
    -1.0, -1.0, -1.0, 0.0, 0.0, 1.0, // Top-right
     1.0, -1.0, -1.0, 0.0, 0.0, 1.0, // Top-left
     1.0, -1.0,  1.0, 0.0, 0.0, 1.0, // Bottom-left
     1.0, -1.0,  1.0, 0.0, 0.0, 1.0, // Bottom-left
    -1.0, -1.0,  1.0, 0.0, 0.0, 1.0, // Bottom-right
    -1.0, -1.0, -1.0, 0.0, 0.0, 1.0, // Top-right

    // Top face
    -1.0,  1.0,  1.0, 0.0, 0.0, 1.0, // Top-left
     1.0,  1.0,  1.0, 0.0, 0.0, 1.0, // Top-right
     1.0,  1.0, -1.0, 0.0, 0.0, 1.0, // Bottom-right
     1.0,  1.0, -1.0, 0.0, 0.0, 1.0, // Bottom-right
    -1.0,  1.0, -1.0, 0.0, 0.0, 1.0, // Bottom-left
    -1.0,  1.0,  1.0, 0.0, 0.0, 1.0, // Top-left
];

  let vertex_source = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    layout (location = 1) in vec3 aNormal;

    out vec3 Normal;
    out vec3 FragPos;

    uniform mat4 model;
    uniform mat4 view;
    uniform mat4 projection;

    void main() {
        FragPos = vec3(model * vec4(aPos, 1.0));
        Normal = mat3(transpose(inverse(model))) * aNormal;

        gl_Position = projection * view * vec4(FragPos, 1.0);
    }
  "#;

  let fragment_source = r#"
    #version 330 core
    out vec4 FragColor;

    in vec3 Normal;
    in vec3 FragPos;

    uniform vec3 lightPos; // Position of the light source
    uniform vec3 viewPos;  // Position of the camera
    uniform vec3 lightColor;
    uniform vec3 objectColor;

    void main() {

        // Ambient lighting
        float ambientStrength = 0.1;
        vec3 ambient = ambientStrength * lightColor;

        // Diffuse lighting
        vec3 norm = normalize(Normal);
        vec3 lightDir = normalize(lightPos - FragPos);
        float diff = max(dot(norm, lightDir), 0.0);
        vec3 diffuse = diff * lightColor;

        vec3 result = (ambient + diffuse) * objectColor;
        FragColor = vec4(result, 1.0);
    }
  "#;

  let shader = Shader::from_source(vertex_source, fragment_source);
  let mut renderer = Renderer::new(&vertices);

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

    renderer.render(&shader, vertices.len(), &model, &view, &projection);

    window.swap_buffers();
    process_input(&mut window);
  }
}

fn process_input(window: &mut glfw::Window) {
  if window.get_key(Key::Escape) == Action::Press {
    window.set_should_close(true);
  }
}