pub use self::simple_rendering::{
  ConsoleRenderer,
  Renderer
};

pub mod simple_rendering {
  use state::{
    GameState,
    Pos
  };

  pub trait Renderer {
    fn render(&self, state: &GameState);
  }

  pub struct ConsoleRenderer {
    pub nothing: i32
  }

  impl Renderer for ConsoleRenderer {
    fn render(&self, state: &GameState) {
      println!("Ya pos is ({}, {})", state.man_pos.x, state.man_pos.y);
    }
  }
}
