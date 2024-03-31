use std::{mem::size_of, ptr};
use gl::{types::{GLsizei, GLsizeiptr, GLvoid}, BindBuffer, BindVertexArray, BufferData, DeleteBuffers, DeleteVertexArrays, EnableVertexAttribArray, GenBuffers, GenVertexArrays, VertexAttribPointer, ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER, STATIC_DRAW };
use image::RgbaImage;
use nalgebra::Matrix4;

use super::shader::Shader;

pub struct RenderObject {
  render_context : RenderContext,
}

impl RenderObject {
  pub fn new(vertices: Vec<f32>, indices: Vec<u32>, shader: Shader, texture: Option<RgbaImage>) -> Self {
    let index_count = vertices.len() as i32;
    RenderObject { 
      render_context : RenderContext::new(vertices, indices, shader, index_count, texture),
    }
  }

  pub fn draw(
    &mut self, model: &Matrix4<f32>, view: &Matrix4<f32>, projection  : &Matrix4<f32>) { 
      self.render_context.draw(model, view, projection) 
    }
}

struct RenderContext {
  shader      : Shader,
  texture     : Option<RgbaImage>, 
  index_count : i32,
  vao         : u32,
  vbo         : u32,
  ebo         : u32,
}

impl RenderContext {

  fn new(vertices: Vec<f32>, indices: Vec<u32>, shader: Shader, index_count: i32, texture: Option<RgbaImage>) -> Self {
    
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

    RenderContext { vao, vbo, ebo, shader, index_count, texture }
  }

  pub fn draw(&mut self, model: &Matrix4<f32>, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
    unsafe {

      self.shader.use_program();
      self.shader.set_mat4("model", model);
      self.shader.set_mat4("view", view);
      self.shader.set_mat4("projection", projection);
      self.shader.render(self.vao, self.index_count);
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