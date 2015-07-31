pub use self::state::{
  Pos,
  GameState
};

mod state {
  pub struct Pos {
    pub x: f32,
    pub y: f32
  }

  pub struct GameState {
    pub man_pos: Pos
  }
}
