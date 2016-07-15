use specs;
use time::Duration;
use engine;

use std::net::SocketAddr;
use std::collections::HashMap;

use world::PlayerAspect;

use engine::player::connection::ConnectEvent;
use itertools::Itertools;

pub struct HealthyEvent(SocketAddr);

impl HealthyEvent {
  pub fn new(address: SocketAddr) -> HealthyEvent {
    HealthyEvent(address)
  }

  pub fn address(&self) -> &SocketAddr {
    let &HealthyEvent(ref addr) = self;
    addr
  }
}

/**
 * Accepts address-specific health events to update player's connection status
 *
 * Inputs: HealthyEvents
 * Outputs: Players, ConnectEvents
 */
pub struct System;

impl System {
  pub fn new() -> System {
    System
  }
}

impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, delta: engine::Delta) {
    use specs::Join;

    let (entities, mut players, mut healthy_events, mut connect_events) = arg.fetch(|w| {
      (w.entities(),
       w.write::<PlayerAspect>(),
       w.write_resource::<Vec<HealthyEvent>>(),
       w.write_resource::<Vec<ConnectEvent>>())
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
