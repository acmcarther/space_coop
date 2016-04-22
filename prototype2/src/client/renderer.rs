use common::world::ClientWorld;

pub struct Renderer;

impl Renderer {
  pub fn new() -> Renderer {
    Renderer
  }

  pub fn render_world(&mut self, world: &Option<&ClientWorld>) {
    match world {
      &Some(world) => {
        println!("world: {:?}",world)
      }
      &None => println!("No World")
    }
  }
}
