use aspects::{CollisionAspect, ControllerAspect, PlayerAspect};
use common::ecs::aspects::{DisabledAspect, PhysicalAspect, RenderAspect, SynchronizedAspect};
use common::protocol::ServerNetworkEvent;
use network::{ConnectEvent, OutboundEvent};
use pubsub::{PubSubStore, SubscriberToken};
use specs;
use state::Delta;
use std::collections::HashMap;

/**
 * Manages connecting and disconnecting player events. Can update and create new players.
 *
 * Players and their associated entities have a lot of creation-time dependencies.
 *
 * TODO(acmcarther): Refactor this whole implementation, its really messy
 *
 * Input: Players, Controllers, ConnectEvent
 * Output: Players, Controllers, Collisions, Disableds, Renders, Physicals
 */
pub struct System {
  connection_event_sub_token: SubscriberToken<ConnectEvent>,
}
declare_dependencies!(System,
                      [::network::DistributionSystem, ::health_check::System]);
standalone_installer_from_new!(System, Delta);

impl System {
  pub fn new(world: &mut specs::World) -> System {
    System { connection_event_sub_token: world.register_subscriber::<ConnectEvent>() }
  }
}

impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, delta: Delta) {
    use itertools::Itertools;
    use specs::Join;

    let (mut player,
         entities,
         mut synchronized,
         mut outbound,
         mut events,
         mut controller,
         mut collision,
         mut disabled,
         mut render,
         mut physical) = arg.fetch(|w| {
      (w.write::<PlayerAspect>(),
       w.entities(),
       w.write::<SynchronizedAspect>(),
       w.fetch_publisher::<OutboundEvent>(),
       w.fetch_subscriber(&self.connection_event_sub_token).collected(),
       w.write::<ControllerAspect>(),
       w.write::<CollisionAspect>(),
       w.write::<DisabledAspect>(),
       w.write::<RenderAspect>(),
       w.write::<PhysicalAspect>())
    });

    // Build synchro -> entity map and our set of synchros
    let mut synchro_to_entity = HashMap::new();
    (&entities, &synchronized).iter().foreach(|(ent, synchro)| {
      synchro_to_entity.insert(synchro.clone(), ent.clone());
    });

    events.drain(..).foreach(|e| {
      match &e {
        &ConnectEvent::Connect(addr) => {
          // TODO: Fix ugly code
          // TODO: Make more efficient, this is currently a linear search
          if let Some((current_player, controller)) = (&mut player, &mut controller)
            .iter()
            .filter(|&(ref player, _)| player.address == addr)
            .next() {
            current_player.connected = true;
            current_player.last_msg = delta.now;
            disabled.remove(synchro_to_entity.get(&controller.subject).unwrap().clone());
            // Dodging borrow checker, by returning instead of else-ing
            outbound.push(OutboundEvent::Directed {
              dest: addr,
              event: ServerNetworkEvent::Connected,
            });
            return;
          } else {
            // self.new_player_addr.send(addr);
          }

          // Else: no existing entity
          let player_ent = arg.create();
          let object_ent = arg.create();
          let object_synchro = SynchronizedAspect::new();
          synchronized.insert(player_ent.clone(), SynchronizedAspect::new());
          synchronized.insert(object_ent.clone(), object_synchro.clone());

          player.insert(player_ent.clone(),
                        PlayerAspect {
                          address: addr,
                          last_msg: delta.now,
                          connected: true,
                        });

          controller.insert(player_ent.clone(),
                            ControllerAspect { subject: object_synchro.clone() });

          render.insert(object_ent.clone(), RenderAspect::new());
          physical.insert(object_ent.clone(),
                          PhysicalAspect::new((0.0, 0.0, 5.0), (0.0, 0.0, 0.0), false));
          collision.insert(object_ent.clone(), CollisionAspect::new());
          outbound.push(OutboundEvent::Directed {
            dest: addr,
            event: ServerNetworkEvent::Connected,
          })
        },
        &ConnectEvent::Disconnect(addr) => {
          println!("disconnect event");
          // TODO: Fix ugly code
          // TODO: Make more efficient, this is currently a linear search
          if let Some((current_player, controller)) = (&mut player, &mut controller)
            .iter()
            .filter(|&(ref player, _)| player.address == addr)
            .next() {
            current_player.connected = false;
            disabled.insert(synchro_to_entity.get(&controller.subject).unwrap().clone(),
                            DisabledAspect::default());
            // Dodging borrow checker, by returning instead of else-ing
            outbound.push(OutboundEvent::Directed {
              dest: addr,
              event: ServerNetworkEvent::Disconnected,
            });
            return;
          }
          // Else: no existing entity
          outbound.push(OutboundEvent::Directed {
            dest: addr,
            event: ServerNetworkEvent::Error("Tried to disconnect, but not connected to server"
              .to_owned()),
          });
        },
      }
    });
  }
}
