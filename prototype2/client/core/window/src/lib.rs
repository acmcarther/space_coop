extern crate specs;
extern crate glutin;
extern crate itertools;
extern crate pubsub;
extern crate client_state as state;
extern crate automatic_system_installer;

use state::Delta;
use pubsub::PubSubStore;

/**
 * Dumps window inputs from glutin
 *
 * This service is currently unused because of an osx issue forcing polling to happen on the main
 * thread.
 *
 * NOTE: This system is unused -- its essentially reimplemented verbatim in the engine in the main
 * thread, due to a bug on macos in intercepting window events off the main thread. See issue #55.
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
