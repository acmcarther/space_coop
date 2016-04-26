use client::renderer::Renderer;
use common::world::ClientWorld;

pub struct ConsoleRenderer;

impl ConsoleRenderer {
  pub fn new() -> ConsoleRenderer {
    ConsoleRenderer
  }
}

impl Renderer for ConsoleRenderer {
  fn render_world(&mut self, world: &Option<&ClientWorld>) {
    match world {
      &Some(world) => {
        println!("world: {:?}",world)
      }
      &None => println!("No World")
    }
  }
}
