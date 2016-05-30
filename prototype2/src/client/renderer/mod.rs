pub mod console;
pub mod opengl;

use cgmath::Quaternion;

use common::world::ClientWorld;

/**
 * A trait for conveying the state of the world to the user
 */
pub trait Renderer {
  fn render_world(&mut self, world: &Option<&ClientWorld>, camera_pos: &(f32, f32, f32), camera_orientation: &Quaternion<f32>);
}
