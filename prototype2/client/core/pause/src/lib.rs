extern crate specs;
extern crate glutin;
extern crate itertools;
extern crate pubsub;
extern crate state;

use pubsub::{PubSubStore, SubscriberToken};

use state::Delta;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PauseState {
  Paused,
  NotPaused,
}

impl PauseState {
  pub fn toggled(&self) -> PauseState {
    match self {
      &PauseState::Paused => PauseState::NotPaused,
      &PauseState::NotPaused => PauseState::Paused,
    }
  }
}

pub struct System {
  window_event_sub_token: SubscriberToken<glutin::Event>,
}

impl System {
  pub fn new(world: &mut specs::World) -> System {
    world.add_resource::<PauseState>(PauseState::NotPaused);

    System { window_event_sub_token: world.register_subscriber::<glutin::Event>() }
  }

  pub fn name() -> &'static str {
    "pause::system"
  }
}

impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use itertools::Itertools;
    use glutin::VirtualKeyCode::Escape;
    use glutin::ElementState;
    use glutin::Event::KeyboardInput;

    let (mut glutin_events, mut pause_state) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.window_event_sub_token).collected(),
       w.write_resource::<PauseState>())
    });


    glutin_events.drain(..).foreach(|e| {
      match e {
        KeyboardInput(ElementState::Pressed, _, Some(Escape)) => {
          *pause_state = pause_state.toggled()
        },
        _ => {}, // I threw it on the ground
      }
    });
  }
}
