use std::ffi::{CStr, CString};
use std::mem::size_of;
use std::{ptr, str};

use gl::types::{GLenum, GLsizei};
use gl::{ActiveTexture, AttachShader, BindVertexArray, CompileShader, CreateProgram, CreateShader, DeleteProgram, DeleteShader, DrawElements, EnableVertexAttribArray, GetUniformLocation, LinkProgram, ShaderSource, UniformMatrix4fv, UseProgram, VertexAttribPointer, FRAGMENT_SHADER, TRIANGLES, UNSIGNED_INT, VERTEX_SHADER};
use image::RgbaImage;
use nalgebra::Matrix4;

pub struct Shader{
  id: u32
}

impl Shader {
  
  pub fn from_source(vertex_source: &str, fragment_source: &str, ) -> Shader {

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

  pub fn set_mat4(&self, name: &str, mat: &Matrix4<f32>) {
    unsafe {
      let loc = GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
      UniformMatrix4fv(loc, 1, gl::FALSE, mat.as_ptr())
    }
  }

  pub fn use_program(&self) {
    unsafe {
      UseProgram(self.id);
    }
  }

  pub fn render(& self, vao: u32, index_count: i32) {
    unsafe {

      BindVertexArray(vao);

      // Position attribute
      //let stride = (6 * size_of::<f32>()) as GLsizei;
      let stride = (5 * size_of::<f32>()) as GLsizei;
      VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, 0 as *const _);
      EnableVertexAttribArray(0);

      // Texture Coordinate Attribute
      VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * size_of::<f32>()) as *const _);
      EnableVertexAttribArray(1);

      DrawElements(
        TRIANGLES,
        index_count as i32,
        UNSIGNED_INT,
        ptr::null()
      );

      BindVertexArray(0);
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