use specs;
use engine;
use common::protocol::ClientNetworkEvent;
use time::{self, Duration, Tm};

/**
 * Useful for Debug
 */
pub struct System {
  next_keepalive_time: Tm,
}

impl System {
  pub fn new() -> System {
    System { next_keepalive_time: time::now() }
  }
}

#[allow(unused_imports, unused_variables)]
impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, delta: engine::Delta) {
    use specs::Join;
    use itertools::Itertools;

    let (mut outbound_events,) = arg.fetch(|w| (w.write_resource::<Vec<ClientNetworkEvent>>(),));

    if delta.now > self.next_keepalive_time {
      outbound_events.push(ClientNetworkEvent::KeepAlive);
      self.next_keepalive_time = delta.now + Duration::milliseconds(20);
    }
  }
}
