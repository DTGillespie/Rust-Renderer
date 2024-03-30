use std::{mem::size_of, ptr};

use gl::{types::{GLsizei, GLsizeiptr, GLvoid}, BindBuffer, BindVertexArray, BufferData, DeleteBuffers, DeleteVertexArrays, DrawArrays, EnableVertexAttribArray, GenBuffers, GenVertexArrays, GetUniformLocation, VertexAttribPointer, ARRAY_BUFFER, STATIC_DRAW, TRIANGLES};
use nalgebra::Matrix4;

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

      VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * size_of::<f32>() as GLsizei, ptr::null());
      EnableVertexAttribArray(0);
      BindBuffer(ARRAY_BUFFER, 0);
      BindVertexArray(0);
    }
    Renderer { vao, vbo }
  }

  pub fn render(&mut self, shader: &Shader, vertex_count: usize, model: &Matrix4<f32>, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
    unsafe {

      shader.use_program();
      shader.set_mat4("model", model);
      shader.set_mat4("view", view);
      shader.set_mat4("projection", projection);

      BindVertexArray(self.vao);

      unsafe {

        // Position attribute
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 6 * size_of::<f32>() as GLsizei, ptr::null());
        gl::EnableVertexAttribArray(0);
        
        // Normal attribute
        let normal_offset = 3 * size_of::<f32>() as GLsizei;
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 6 * size_of::<f32>() as GLsizei, normal_offset as *const _);
        gl::EnableVertexAttribArray(1);
      }

      /*
      shader.set_vec3("lightPos", &light_pos);
      shader.set_vec3("viewPos", &camera_pos);
      shader.set_vec3("lightColor", &light_color);
      shader.set_vec3("objectColor", &object_color);
      */

      DrawArrays(TRIANGLES, 0, vertex_count as GLsizei);
      BindVertexArray(0);
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