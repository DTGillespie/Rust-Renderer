use nalgebra::{Matrix4, Point3, Vector3};

pub struct Viewport {
  pub position : Point3<f32>,
  pub target   : Point3<f32>,
  pub up       : Vector3<f32>
}

impl Viewport {
  pub fn new(position: Point3<f32>, target: Point3<f32>, up: Vector3<f32>) -> Self {
    Self { position, target, up }
  }

  pub fn get_view_matrix(&self) -> Matrix4<f32> {
    Matrix4::look_at_rh(&self.position, &self.target, &self.up)
  }
}