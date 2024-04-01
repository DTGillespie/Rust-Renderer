use std::{ffi::CString, mem::size_of, ptr};
use gl::{types::{GLenum, GLint, GLsizei, GLsizeiptr, GLuint, GLvoid}, ActiveTexture, AttachShader, BindBuffer, BindTexture, BindVertexArray, BufferData, CompileShader, CreateProgram, CreateShader, DeleteBuffers, DeleteProgram, DeleteShader, DeleteVertexArrays, DrawElements, EnableVertexAttribArray, GenBuffers, GenTextures, GenVertexArrays, GetUniformLocation, LinkProgram, ShaderSource, TexImage2D, TexParameteri, Uniform1i, UniformMatrix4fv, UseProgram, VertexAttribPointer, ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER, FRAGMENT_SHADER, LINEAR, REPEAT, RGBA, STATIC_DRAW, TEXTURE0, TEXTURE_2D, TEXTURE_MAG_FILTER, TEXTURE_MIN_FILTER, TEXTURE_WRAP_R, TEXTURE_WRAP_S, TEXTURE_WRAP_T, TRIANGLES, UNSIGNED_INT, VERTEX_SHADER };
use image::RgbaImage;
use nalgebra::Matrix4;

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

  fn get_uniform_location(&self, name: &str) -> i32 {
    let c_str_name = CString::new(name).expect("Failed to convert string to CString");
    unsafe {
        gl::GetUniformLocation(self.id, c_str_name.as_ptr())
    }
  }

  fn set_mat4(&self, name: &str, mat: &Matrix4<f32>) {
    unsafe {
      let loc = GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
      UniformMatrix4fv(loc, 1, gl::FALSE, mat.as_ptr())
    }
  }

  fn use_program(&self) {
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

struct RenderContext {
  shader      : Shader,
  texture_id  : u32, 
  index_count : i32,
  vao         : u32,
  vbo         : u32,
  ebo         : u32,
}

impl RenderContext {

  fn new(vertices: Vec<f32>, indices: Vec<u32>, shader: Shader, index_count: i32, texture: Option<RgbaImage>) -> Self {
    
    let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
    let mut texture_id: GLuint = 0;

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

      // Attribute Stride
      let stride = (5 * std::mem::size_of::<f32>()) as gl::types::GLsizei;

      // Vertex Attribtues
      //VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * size_of::<f32>() as GLsizei, ptr::null());
      //EnableVertexAttribArray(0);
      VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, 0 as *const _);
      EnableVertexAttribArray(0);

      if let Some(texture) = texture {

        GenTextures(1, &mut texture_id);
        BindTexture(TEXTURE_2D, texture_id);

        TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT as GLint);
        TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT as GLint);
        TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as GLint);
        TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as GLint);

        Uniform1i(shader.get_uniform_location("texture1"), 0);
        
        let (width, height) = (texture.width() as i32, texture.height() as i32);
        let raw_data = texture.into_raw();
        let data = raw_data.as_slice();
    
        unsafe {
          gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as GLint,
            width,
            height,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const GLvoid,
          );

          gl::GenerateMipmap(gl::TEXTURE_2D);      
        }

        BindBuffer(ARRAY_BUFFER, 0);
        BindVertexArray(0);
      }

    }

    RenderContext { vao, vbo, ebo, shader, index_count, texture_id }
  }

  fn draw(&mut self, model: &Matrix4<f32>, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
    unsafe {

      self.shader.use_program();
      self.shader.set_mat4("model", model);
      self.shader.set_mat4("view", view);
      self.shader.set_mat4("projection", projection);

      unsafe {

        BindVertexArray(self.vao);
  
        // Position attribute
        //let stride = (6 * size_of::<f32>()) as GLsizei;
        let stride = (5 * size_of::<f32>()) as GLsizei;
        VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, 0 as *const _);
        EnableVertexAttribArray(0);
  
        // Texture Coordinate Attribute
        VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * size_of::<f32>()) as *const _);
        EnableVertexAttribArray(1);

        ActiveTexture(TEXTURE0);
        BindTexture(TEXTURE_2D, self.texture_id);
  
        DrawElements(
          TRIANGLES,
          self.index_count,
          UNSIGNED_INT,
          ptr::null()
        );
  
        BindVertexArray(0);
      }

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