pub mod window;

use itertools::Itertools;
use specs;
use state::Delta;
use pubsub::{PubSubStore, SubscriberToken};
use common::protocol::ClientNetworkEvent;

pub struct Initializer {
  system_installers: Vec<Box<FnMut(&mut specs::Planner<Delta>) + Send>>,
}

impl Initializer {
  pub fn new() -> Initializer {
    Initializer { system_installers: Vec::new() }
  }

  pub fn add_system<T: specs::System<Delta> + 'static>(&mut self,
                                                       system: T,
                                                       name: String,
                                                       priority: specs::Priority) {
    use std::ops::Deref;

    let mut consumable_system = Some(system);
    let mut consumable_name = Some(name);
    let mut consumable_priority = Some(priority);
    self.system_installers
      .push(Box::new(move |planner: &mut specs::Planner<Delta>| {
        planner.add_system(consumable_system.take().unwrap(),
                           consumable_name.take().unwrap().deref(),
                           consumable_priority.take().unwrap())
      }));
  }

  pub fn build(self, world: specs::World) -> specs::Planner<Delta> {
    use std::ops::Deref;
    let mut planner = specs::Planner::new(world, 2);

    self.system_installers
      .into_iter()
      .foreach(|mut installer| installer(&mut planner));

    planner
  }
}

pub struct ClientWorld {
  pub planner: specs::Planner<Delta>,
  pub net_subscriber: Option<SubscriberToken<ClientNetworkEvent>>,
}

impl ClientWorld {
  pub fn new() -> ClientWorld {
    ClientWorld {
      planner: specs::Planner::new(specs::World::new(), 1),
      net_subscriber: None,
    }
  }
}
