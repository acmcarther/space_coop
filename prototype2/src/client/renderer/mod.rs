pub mod console;
pub mod opengl;

use common::world::ClientWorld;

pub trait Renderer {
  fn render_world(&mut self, world: &Option<&ClientWorld>);
}
