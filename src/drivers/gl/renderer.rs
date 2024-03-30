use std::mem::size_of;

use gl::{types::{GLsizei, GLsizeiptr, GLvoid}, BindBuffer, BindVertexArray, BufferData, DeleteBuffers, DeleteVertexArrays, DrawArrays, EnableVertexAttribArray, GenBuffers, GenVertexArrays, VertexAttribPointer, ARRAY_BUFFER, STATIC_DRAW, TRIANGLES};

use super::shader::Shader;

pub struct Renderer {
  vao: u32,
  vbo: u32
}

impl Renderer {

  pub fn new(vertices: &[f32]) -> Renderer {
    
    let (mut vao, mut vbo) = (0, 0);
    unsafe {
      GenVertexArrays(1, &mut vao);
      GenBuffers(1, &mut vbo);
      BindVertexArray(vao);
      BindBuffer(ARRAY_BUFFER, vbo);
      
      BufferData(ARRAY_BUFFER, 
        (vertices.len() * size_of::<f32>()) as GLsizeiptr,
        vertices.as_ptr() as *const GLvoid,
        STATIC_DRAW
      );

      VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * size_of::<f32>() as GLsizei, std::ptr::null());
      EnableVertexAttribArray(0);
      BindBuffer(ARRAY_BUFFER, 0);
      BindVertexArray(0);
    }
    Renderer { vao, vbo }
  }

  pub fn render(&mut self, shader: &Shader) {
    unsafe {
      shader.use_program();
      BindVertexArray(self.vao);
      DrawArrays(TRIANGLES, 0, 3);
    }
  }
}

impl Drop for Renderer {
  fn drop(&mut self) {
    unsafe {
      DeleteVertexArrays(1, &mut self.vao);
      DeleteBuffers(1, &mut self.vbo);
    }
  }
}