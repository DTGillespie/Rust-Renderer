use std::ffi::{CStr, CString};
use std::str;

use gl::types::GLenum;
use gl::{AttachShader, CompileShader, CreateProgram, CreateShader, DeleteProgram, DeleteShader, LinkProgram, ShaderSource, UseProgram, FRAGMENT_SHADER, VERTEX_SHADER};

pub struct Shader{
  id: u32
}

impl Shader {
  
  pub fn from_source(vertex_source: &str, fragment_source: &str) -> Shader {

    let compile_shader = |src: &str, ty: GLenum| -> u32 {
      let shader;
      unsafe {
        shader = CreateShader(ty);
        ShaderSource(shader, 1, &CString::new(src).unwrap().as_ptr(), std::ptr::null());
        CompileShader(shader);
      }
      shader
    };

    let vertex_shader = compile_shader(vertex_source, VERTEX_SHADER);
    let fragment_shader = compile_shader(fragment_source, FRAGMENT_SHADER);

    let program;
    unsafe {
      program = CreateProgram();
      AttachShader(program, vertex_shader);
      AttachShader(program, fragment_shader);
      LinkProgram(program);

      DeleteShader(vertex_shader);
      DeleteShader(fragment_shader);
    }

    Shader { id: program }
  }

  pub fn use_program(&self) {
    unsafe {
      UseProgram(self.id);
    }
  }
}

impl Drop for Shader {
  fn drop(&mut self) {
    unsafe {
      DeleteProgram(self.id);
    }
  }
}