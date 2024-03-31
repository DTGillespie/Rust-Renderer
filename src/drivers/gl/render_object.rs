use std::{mem::size_of, ptr};
use gl::{types::{GLsizei, GLsizeiptr, GLvoid}, BindBuffer, BindVertexArray, BufferData, DeleteBuffers, DeleteVertexArrays, DrawArrays, DrawElements, EnableVertexAttribArray, GenBuffers, GenVertexArrays, GetUniformLocation, VertexAttribPointer, ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER, STATIC_DRAW, TRIANGLES, UNSIGNED_INT};
use nalgebra::Matrix4;

use super::shader::Shader;

pub struct RenderObject {
  render_context: RenderContext
}

impl RenderObject {
  pub fn new(vertices: &[f32], indices: &[u32]) -> Self {
    RenderObject { render_context: RenderContext::new(vertices, indices) }
  }

  pub fn render(
    &mut self, 
    shader      : &Shader, 
    index_count : usize, 
    model       : &Matrix4<f32>, 
    view        : &Matrix4<f32>, 
    projection  : &Matrix4<f32>
  ) { self.render_context.draw(shader, index_count, model, view, projection) }
}

struct RenderContext {
  vao: u32,
  vbo: u32,
  ebo: u32,
}

impl RenderContext {

  fn new(vertices: &[f32], indices: &[u32]) -> Self {
    
    let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
    unsafe {

      GenVertexArrays(1, &mut vao);
      GenBuffers(1, &mut vbo);
      GenBuffers(1, &mut ebo);

      BindVertexArray(vao);
      
      // VBO
      BindBuffer(ARRAY_BUFFER, vbo);
      BufferData(ARRAY_BUFFER, 
        (vertices.len() * size_of::<f32>()) as GLsizeiptr,
        vertices.as_ptr() as *const GLvoid,
        STATIC_DRAW
      );

      // EBO
      BindBuffer(ELEMENT_ARRAY_BUFFER, ebo);
      BufferData(
        ELEMENT_ARRAY_BUFFER,
        (indices.len() * size_of::<u32>()) as GLsizeiptr,
        indices.as_ptr() as *const GLvoid,
        STATIC_DRAW
      );

      // Vertex Attribtues
      VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * size_of::<f32>() as GLsizei, ptr::null());
      EnableVertexAttribArray(0);

      BindBuffer(ARRAY_BUFFER, 0);
      BindVertexArray(0);
    }
    RenderContext { vao, vbo, ebo}
  }

  pub fn draw(&mut self, shader: &Shader, index_count: usize, model: &Matrix4<f32>, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
    unsafe {

      shader.use_program();
      shader.set_mat4("model", model);
      shader.set_mat4("view", view);
      shader.set_mat4("projection", projection);

      BindVertexArray(self.vao);

      unsafe {

        // Position attribute
        //let stride = (6 * size_of::<f32>()) as GLsizei;
        let stride = (5 * size_of::<f32>()) as GLsizei;
        VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, 0 as *const _);
        EnableVertexAttribArray(0);

        // Texture Coordinate Attribute
        VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * size_of::<f32>()) as *const _);
        EnableVertexAttribArray(1);

      }

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

impl Drop for RenderContext {
  fn drop(&mut self) {
    unsafe {
      DeleteBuffers(1, &mut self.ebo);
      DeleteBuffers(1, &mut self.vbo);
      DeleteVertexArrays(1, &mut self.vao);
    }
  }
}