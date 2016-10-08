use glutin;
use input::ConsoleEvent;
use pause::PauseState;
use pubsub::{PubSubStore, SubscriberToken};
use specs;
use state::Delta;

pub struct System {
  window_event_sub_token: SubscriberToken<glutin::Event>,
}
// NOTE: This depends on a window emitter that lives in the main thread
declare_dependencies!(System, [::pause::System]);
standalone_installer_from_new!(System, Delta);

impl System {
  pub fn new(world: &mut specs::World) -> System {
    System { window_event_sub_token: world.register_subscriber::<glutin::Event>() }
  }
}

impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use itertools::Itertools;

    let (mut glutin_events, pause_state, mut console_events) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.window_event_sub_token).collected(),
       w.read_resource::<PauseState>(),
       w.fetch_publisher::<ConsoleEvent>())
    });

    if *pause_state == PauseState::Paused {
      glutin_events.drain(..).foreach(|e| {
        console_events.push(ConsoleEvent::from(e));
      });
    }
  }
}
