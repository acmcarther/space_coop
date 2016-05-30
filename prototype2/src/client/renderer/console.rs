use cgmath::Quaternion;

use client::renderer::Renderer;
use common::world::ClientWorld;

/**
 * A very simple renderer that just logs the world to console
 */
pub struct ConsoleRenderer;

impl ConsoleRenderer {
  pub fn new() -> ConsoleRenderer {
    ConsoleRenderer
  }
}

impl Renderer for ConsoleRenderer {
  fn render_world(&mut self, world: &Option<&ClientWorld>, _: &(f32, f32, f32), _: &Quaternion<f32>) {
    match world {
      &Some(world) => {
        println!("world: {:?}",world)
      }
      &None => println!("No World")
    }
  }
}
