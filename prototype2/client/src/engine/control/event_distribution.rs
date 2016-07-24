use specs;
use glutin;

use engine;

use world::ExitFlag;
use glutin::Event::KeyboardInput;
use glutin::Event::MouseMoved;
use engine::control::player::MoveEvent;
use engine::control::camera::CameraMoveEvent;

/**
 * Send the events from the windowing system to event busses
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

    let (mut glutin_events, mut move_events, mut camera_events, mut exit_flag) = arg.fetch(|w| {
        (w.write_resource::<Vec<glutin::Event>>(),
         w.write_resource::<Vec<MoveEvent>>(),
         w.write_resource::<Vec<CameraMoveEvent>>(),
         w.write_resource::<ExitFlag>())
      });

    glutin_events.drain(..).foreach(|e| {
      use glutin::VirtualKeyCode::{A, D, Escape, S, W};
      match e {
        KeyboardInput(_, _, Some(Escape)) => *exit_flag = ExitFlag(true),
        KeyboardInput(_, _, Some(W)) => move_events.push(MoveEvent::Forward),
        KeyboardInput(_, _, Some(A)) => move_events.push(MoveEvent::Left),
        KeyboardInput(_, _, Some(S)) => move_events.push(MoveEvent::Backward),
        KeyboardInput(_, _, Some(D)) => move_events.push(MoveEvent::Right),
        MouseMoved(x, y) => camera_events.push(CameraMoveEvent(x, y)),
        _ => {},
      }
    });
  }
}
