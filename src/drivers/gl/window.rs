extern crate gl;
extern crate glfw;

use std::{env, path::{Path, PathBuf}};

use gl::{Clear, ClearColor, COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT, DEPTH_TEST};
use glfw::{fail_on_errors, Action, Context, Key};
use nalgebra::{Matrix4, Perspective3, Point3, Vector3};

use super::{ render_object::RenderObject, render_object::Shader, utils::{ load_image, load_obj }, viewport::Viewport};

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
  /*
  let vertices: [f32; 40] = [ // Don't think these texture cordinates are implemented yet, so the model is messed up
    // Position       // Texture Coords   // Description
    -1.0, -1.0, -1.0, 0.0, 0.0,           // Back-bottom-left   0
     1.0, -1.0, -1.0, 1.0, 0.0,           // Back-bottom-right  1
     1.0,  1.0, -1.0, 1.0, 1.0,           // Back-top-right     2
    -1.0,  1.0, -1.0, 0.0, 1.0,           // Back-top-left      3
    -1.0, -1.0,  1.0, 0.0, 0.0,           // Front-bottom-left  4
     1.0, -1.0,  1.0, 1.0, 0.0,           // Front-bottom-right 5
     1.0,  1.0,  1.0, 1.0, 1.0,           // Front-top-right    6
    -1.0,  1.0,  1.0, 0.0, 1.0,           // Front-top-left     7
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
*/

  let vertex_source = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    layout (location = 1) in vec2 aTexCoord;

    out vec2 TexCoord;

    uniform mat4 model;
    uniform mat4 view;
    uniform mat4 projection;

    void main() {
        gl_Position = projection * view * model * vec4(aPos, 1.0);
        TexCoord = aTexCoord;
    }
  "#;

  let fragment_source = r#"
    #version 330 core
    out vec4 FragColor;
    
    in vec2 TexCoord;
    
    uniform sampler2D texture1;
    
    void main() {
      FragColor = texture(texture1, TexCoord);
    }
  "#;

  let cwd = env::current_dir().expect("Failed to get current working directory");
  let root_dir = cwd.parent()
                                 .and_then(Path::parent)
                                 .map(PathBuf::from)
                                 .expect("Failed to navigate working directory");

  let texture_path = root_dir.join("assets/test_texture.jpg");
  let texture_path_str = texture_path.to_str().expect("Path contains invalid unicode");
  
  let cube_model_path = root_dir.join("assets/cube.obj");
  let (vertices, indices) = load_obj(cube_model_path)
        .unwrap_or_else(|err| {
            eprintln!("Error loading .obj file: {:?}", err);
            (vec![], vec![]) // Provide default empty vectors if loading fails
        });

  println!("Vertices:");
  for chunk in vertices.chunks(5) {
    println!("Position: ({}, {}, {}), Texture Coords: ({}, {})", chunk[0], chunk[1], chunk[2], chunk[3], chunk[4]);
  }

  println!("Indices:");
  for triangle in indices.chunks(3) {
    println!("Triangle: {}, {}, {}", triangle[0], triangle[1], triangle[2]);
  }
  
  let mut cube = RenderObject::new(
    vertices.to_vec(), 
    indices.to_vec(),
    Shader::from_source(vertex_source, fragment_source),
    Some(load_image(texture_path_str).expect("Failed to load texture")),
  );

  let viewport = Viewport::new(
    Point3::new(0.0, 0.0, 3.0),
    Point3::origin(),
    Vector3::y()
  );

  let aspect_ratio = 1600.0 / 1200.0;
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

    cube.draw(&model, &view, &projection);

    window.swap_buffers();
    process_input(&mut window);
  }
}

fn process_input(window: &mut glfw::Window) {
  if window.get_key(Key::Escape) == Action::Press {
    window.set_should_close(true);
  }
}