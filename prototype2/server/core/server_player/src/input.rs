use specs;
use std::net::SocketAddr;
use std::collections::HashMap;

use common::protocol::ClientEvent;
use common::aspects::{PhysicalAspect, RenderAspect, SynchronizedAspect};
use aspects::{CollisionAspect, ControllerAspect, PlayerAspect};
use state::Delta;
use pubsub::{PubSubStore, SubscriberToken};

#[derive(Debug, Clone)]
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
pub struct System {
  input_event_sub_token: SubscriberToken<InputEvent>,
}

impl System {
  pub fn new(world: &mut specs::World) -> System {
    System { input_event_sub_token: world.register_subscriber() }
  }
}

impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use specs::Join;
    use itertools::Itertools;

    let (mut client_events,
         entities,
         mut synchronized,
         mut render,
         mut collision,
         mut physicals,
         players,
         controllers) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.input_event_sub_token).collected(),
       w.entities(),
       w.write::<SynchronizedAspect>(),
       w.write::<RenderAspect>(),
       w.write::<CollisionAspect>(),
       w.write::<PhysicalAspect>(),
       w.read::<PlayerAspect>(),
       w.read::<ControllerAspect>())
    });

    // Build synchro -> entity map and our set of synchros
    let mut synchro_to_entity = HashMap::new();
    (&entities, &synchronized).iter().foreach(|(ent, synchro)| {
      synchro_to_entity.insert(synchro.clone(), ent.clone());
    });

    // Grab our controller
    // TODO(acmcarther): More robust address -> player association. This is
    // insecure and also inconvient
    let mut address_to_controller = HashMap::new();
    (&players, &controllers)
      .iter()
      .foreach(|(player, controller)| {
        address_to_controller.insert(player.address.clone(), controller.clone());
      });


    client_events.drain(..).foreach(|event| {
      match event.event {
        ClientEvent::SelfMove { x_d, y_d, z_d } => {
          if let Some(controller) = address_to_controller.get(&event.address) {
            let mut physical =
              physicals.get_mut(synchro_to_entity.get(&controller.subject).unwrap().clone())
                .unwrap();
            physical.vel.0 = physical.vel.0 + x_d;
            physical.vel.1 = physical.vel.1 + y_d;
            physical.vel.2 = physical.vel.2 + z_d;
          }
        },
        ClientEvent::CreateEntity => {
          let ent = arg.create();
          synchronized.insert(ent.clone(), SynchronizedAspect::new());
          render.insert(ent.clone(), RenderAspect::new());
          physicals.insert(ent.clone(),
                           PhysicalAspect::new((0.0, 0.0, 5.0), (0.0, 0.0, 0.0), false));
          collision.insert(ent.clone(), CollisionAspect::new());
        },
        ClientEvent::DeleteEntity(synchro) => {
          synchro_to_entity.get(&synchro).map(|e| arg.delete(e.clone()));
        },
        ClientEvent::MutatePhysicalAspect(synchro, new_physical) => {
          synchro_to_entity.get(&synchro).map(|e| physicals.insert(e.clone(), new_physical));
        },
        ClientEvent::MutateRenderAspect(synchro, new_render) => {
          synchro_to_entity.get(&synchro).map(|e| {
            collision.insert(e.clone(), CollisionAspect::from_render(&new_render));
            render.insert(e.clone(), new_render)
          });
        },
      };
    });
  }
}
