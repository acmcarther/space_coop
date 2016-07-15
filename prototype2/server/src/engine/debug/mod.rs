use specs;
use engine;

use world::PlayerAspect;

/**
 * Useful for Debug
 */
pub struct System;

impl System {
  pub fn new() -> System {
    System
  }
}

impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    let (_, _) = arg.fetch(|w| (w.entities(), w.read::<PlayerAspect>()));
  }
}
