use common::world::ClientWorld;

use uuid::Uuid;

use itertools::Itertools;
pub struct Renderer;


impl Renderer {
  pub fn new() -> Renderer {
    Renderer
  }

  //pub fn mut_window(&mut self) -> &mut glutin::Window { &mut self.window }

  pub fn render_world(&mut self, world: &Option<&ClientWorld>) {
    match world {
      &Some(world) => {
        println!("world: {:?}",world)
      }
      &None => println!("No World")
    }
  }
}
