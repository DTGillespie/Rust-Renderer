use std::fs::File;
use std::io::{self, BufRead};
use std::{error::Error, path::Path};
use image::io::Reader as ImageReader;
use image::RgbaImage;

pub struct ObjData {
  vertices: Vec<f32>,
  tex_coords: Vec<f32>,
  indices: Vec<u32>,
}

pub fn load_image(path: &str) -> Result<RgbaImage, Box<dyn Error>> {
  let img = ImageReader::open(Path::new(path))?
    .decode()?
    .flipv()
    .to_rgba8();
  Ok(img)
}

pub fn read_obj_file(file_path: &str) -> io::Result<ObjData> {

  let path = Path::new(file_path);
  let file = File::open(path)?;
  let reader = io::BufReader::new(file);

  let mut vertices = Vec::new();
  let mut tex_coords = Vec::new();
  let mut temp_indices = Vec::new();
  let mut indices = Vec::new();

  for line in reader.lines() {
      let line = line?;
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.is_empty() {
          continue;
      }

      match parts[0] {
          "v" => {
              // Vertex position
              let pos: Vec<f32> = parts[1..]
                  .iter()
                  .map(|p| p.parse().unwrap())
                  .collect();
              vertices.extend(pos);
          }
          "vt" => {
              // Texture coordinate
              let tex: Vec<f32> = parts[1..]
                  .iter()
                  .map(|t| t.parse().unwrap())
                  .collect();
              tex_coords.extend(tex);
          }
          "f" => {
              // Face definition
              for part in &parts[1..] {
                  let indices: Vec<&str> = part.split('/').collect();
                  let vertex_index: u32 = indices[0].parse().unwrap();
                  // OBJ file indices start at 1, so we adjust them to start at 0
                  temp_indices.push(vertex_index - 1);

                  if indices.len() > 1 && !indices[1].is_empty() {
                      let tex_index: u32 = indices[1].parse().unwrap();
                      indices.push(tex_index - 1);
                  }
              }
          }
          _ => {}
      }
  }

  // Convert vertex indices to the format required by the question.
  // Here, assuming each face is a triangle for simplicity. If the OBJ uses quads or other forms, more processing is needed.
  for chunk in temp_indices.chunks(3) {
      if chunk.len() == 3 {
          indices.extend_from_slice(chunk);
      }
  }

  Ok(ObjData {
      vertices,
      tex_coords,
      indices,
  })
}
