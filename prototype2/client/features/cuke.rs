#[macro_use]
extern crate cucumber;
extern crate specs;
extern crate itertools;
extern crate glutin;
extern crate client_state as state;
extern crate pubsub;
extern crate common;
extern crate client;
extern crate camera;
extern crate mouse_lock;
extern crate pause;
extern crate time;
#[macro_use]
extern crate lazy_static;
extern crate automatic_system_installer;

mod step_definitions;
mod support;

use step_definitions::engine_steps;
use step_definitions::event_steps;
use step_definitions::window_steps;
use step_definitions::camera_steps;
use step_definitions::pause_steps;
use step_definitions::net_event_steps;
use support::ClientWorld;

#[test]
fn cuke() {
  cucumber::start(ClientWorld::new(),
                  &[&engine_steps::register_steps,
                    &event_steps::register_steps,
                    &window_steps::register_steps,
                    &camera_steps::register_steps,
                    &net_event_steps::register_steps,
                    &pause_steps::register_steps]);
}

fn main() {
  cuke()
}
