extern crate itertools;
extern crate specs;
extern crate pubsub;
extern crate glutin;
extern crate client_state as state;
extern crate pause;

use pubsub::{PubSubStore, SubscriberToken};
use state::Delta;
use pause::PauseState;

#[derive(Clone, Debug, PartialEq)]
pub struct RelativeMouseMovementEvent {
  pub x: i32,
  pub y: i32,
}

pub struct System {
  window_event_sub_token: SubscriberToken<glutin::Event>,
}

impl System {
  pub fn new(world: &mut specs::World) -> System {
    System { window_event_sub_token: world.register_subscriber::<glutin::Event>() }
  }

  pub fn name() -> &'static str {
    "mouse_lock::System"
  }
}

impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use itertools::Itertools;
    use glutin::Event::MouseMoved;

    let (mut glutin_events, pause_state, window, mut relative_mouse_movement_events) =
      arg.fetch(|w| {
        (w.fetch_subscriber(&self.window_event_sub_token).collected(),
         w.read_resource::<PauseState>(),
         w.write_resource::<glutin::Window>(),
         w.fetch_publisher::<RelativeMouseMovementEvent>())
      });

    if *pause_state != PauseState::Paused {
      glutin_events.drain(..).foreach(|e| {
        match e {
          MouseMoved(x, y) => {
            // Move the mouse back to the middle of the window
            let (wx, wy) = window.get_position().unwrap();
            let (ox, oy) = window.get_outer_size().unwrap();
            let (middle_x, middle_y) = ((wx + ox as i32 / 2), (wy + oy as i32 / 2));
            window.set_cursor_position(middle_x, middle_y).unwrap();

            relative_mouse_movement_events.push(RelativeMouseMovementEvent {
              x: x - middle_x,
              y: y - middle_y,
            });
          },
          _ => {}, // It's goin' on the ground
        }
      });
    }
  }
}
