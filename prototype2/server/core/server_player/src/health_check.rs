use specs;
use time::Duration;

use std::collections::HashMap;

use aspects::PlayerAspect;
use state::Delta;

use itertools::Itertools;
use pubsub::{PubSubStore, SubscriberToken};
use network::{ConnectEvent, HealthyEvent};

/**
 * Accepts address-specific health events to update player's connection status
 *
 * Inputs: HealthyEvents
 * Outputs: Players, ConnectEvents
 */
pub struct System {
  healthy_event_sub_token: SubscriberToken<HealthyEvent>,
}
declare_dependencies!(System, [::network::DistributionSystem]);
standalone_installer_from_new!(System, Delta);

impl System {
  pub fn new(world: &mut specs::World) -> System {
    System { healthy_event_sub_token: world.register_subscriber() }
  }
}

impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, delta: Delta) {
    use specs::Join;

    let (entities, mut players, mut healthy_events, mut connect_events) = arg.fetch(|w| {
      (w.entities(),
       w.write::<PlayerAspect>(),
       w.fetch_subscriber(&self.healthy_event_sub_token).collected(),
       w.fetch_publisher::<ConnectEvent>())
    });

    // Build address to entity mapping for convenience
    let mut addr_to_entity = HashMap::new();
    (&entities, &players)
      .iter()
      .filter(|&(_, ref player)| player.connected)
      .foreach(|(entity, player)| {
        addr_to_entity.insert(player.address.clone(), entity.clone());
      });

    // Set all affected players last_msg to now
    healthy_events.drain(..)
      .filter_map(|event| addr_to_entity.get(event.address()).map(|v| v.clone()))
      .foreach(|entity| {
        players.get_mut(entity).unwrap().last_msg = delta.now.clone();
      });

    // Disconnect any dead players
    players.iter()
      .filter(|&player| player.connected && delta.now - player.last_msg > Duration::seconds(3))
      .foreach(|player| {
        connect_events.push(ConnectEvent::Disconnect(player.address));
      });
  }
}
