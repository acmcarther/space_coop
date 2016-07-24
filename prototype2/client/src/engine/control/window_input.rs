use specs;
use glutin;

use engine;

/**
 * Dumps window inputs from glutin
 */
pub struct System;

impl System {
  pub fn new() -> System {
    System
  }
}

#[allow(unused_imports, unused_variables)]
impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    use specs::Join;
    use itertools::Itertools;

    let (window, mut glutin_events) =
      arg.fetch(|w| {
        (w.write_resource::<glutin::Window>(), w.write_resource::<Vec<glutin::Event>>())
      });

    glutin_events.extend(window.poll_events());
  }
}
