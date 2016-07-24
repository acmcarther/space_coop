use specs;
use engine;
use std::net::SocketAddr;
use std::collections::HashMap;

use common::protocol::ClientEvent;
use common::world::{PhysicalAspect, SynchronizedAspect};
use world::{ControllerAspect, PlayerAspect};

pub struct InputEvent {
  pub address: SocketAddr,
  pub event: ClientEvent,
}

impl InputEvent {
  pub fn new(address: SocketAddr, event: ClientEvent) -> InputEvent {
    InputEvent {
      address: address,
      event: event,
    }
  }
}

/**
 * Handles input events for players
 *
 * Inputs: ClientEvent, Player
 *
 */
pub struct System;

impl System {
  pub fn new() -> System {
    System
  }
}

impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    use specs::Join;
    use itertools::Itertools;

    let (mut client_events, entities, synchronized, players, controllers, mut physicals) =
      arg.fetch(|w| {
        (w.write_resource::<Vec<InputEvent>>(),
         w.entities(),
         w.read::<SynchronizedAspect>(),
         w.read::<PlayerAspect>(),
         w.read::<ControllerAspect>(),
         w.write::<PhysicalAspect>())
      });

    // Build synchro -> entity map and our set of synchros
    let mut synchro_to_entity = HashMap::new();
    (&entities, &synchronized).iter().foreach(|(ent, synchro)| {
      synchro_to_entity.insert(synchro.clone(), ent.clone());
    });

    client_events.drain(..).foreach(|event| {
      // Get controllable entity associated with the message sender
      // TODO(acmcarther): More robust address -> player association. This is
      // insecure and also
      // inconvient
      if let Some((_, controller)) = (&players, &controllers)
        .iter()
        .filter(|&(ref player, _)| player.address == event.address)
        .next() {
        match event.event {
          ClientEvent::SelfMove { x_d, y_d, z_d } => {
            let mut physical =
              physicals.get_mut(synchro_to_entity.get(&controller.subject).unwrap().clone())
                .unwrap();
            physical.vel.0 = physical.vel.0 + x_d;
            physical.vel.1 = physical.vel.1 + y_d;
            physical.vel.2 = physical.vel.2 + z_d;
          },
        }
      }
    });
  }
}
