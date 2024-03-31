use std::{error::Error, path::Path};
use image::io::Reader as ImageReader;
use image::RgbaImage;

pub fn load_image(path: &str) -> Result<RgbaImage, Box<dyn Error>> {
  let img = ImageReader::open(Path::new(path))?
    .decode()?
    .flipv()
    .to_rgba8();
  Ok(img)
}