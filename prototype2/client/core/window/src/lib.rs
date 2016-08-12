extern crate specs;
extern crate glutin;
extern crate itertools;
extern crate pubsub;
extern crate client_state as state;

use state::Delta;
use pubsub::PubSubStore;

/**
 * Dumps window inputs from glutin
 *
 * This service is currently unused because of an osx issue forcing polling to happen on the main
 * thread.
 */
pub struct System;

impl System {
  pub fn new() -> System {
    System
  }
}

#[allow(unused_imports, unused_variables)]
impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use specs::Join;
    use itertools::Itertools;

    let (window, mut glutin_events) =
      arg.fetch(|w| (w.write_resource::<glutin::Window>(), w.fetch_publisher::<glutin::Event>()));

    window.poll_events().into_iter().foreach(|e| glutin_events.push(e));
  }
}
