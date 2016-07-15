use std::net::SocketAddr;

use world::{
  PlayerAspect,
  ControllerAspect,
  CollisionAspect,
};

use common::world::{
  RenderAspect,
  PhysicalAspect,
  DisabledAspect,
};

use common::protocol::{ServerNetworkEvent};
use protocol::OutboundEvent;

use specs;
use engine;

pub enum ConnectEvent {
  Connect(SocketAddr),
  Disconnect(SocketAddr)
}

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
pub struct System;

impl System {
  pub fn new() -> System {
    System
  }
}

impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, delta: engine::Delta) {
    use itertools::Itertools;
    use specs::Join;

    let (mut player, mut outbound, mut events, mut controller, mut collision, mut disabled, mut render, mut physical) = arg.fetch(|w| {
       (w.write::<PlayerAspect>(),
        w.write_resource::<Vec<OutboundEvent>>(),
        w.write_resource::<Vec<ConnectEvent>>(),
        w.write::<ControllerAspect>(),
        w.write::<CollisionAspect>(),
        w.write::<DisabledAspect>(),
        w.write::<RenderAspect>(),
        w.write::<PhysicalAspect>())
    });

    events.drain(..).foreach(|e| match &e {
      &ConnectEvent::Connect(addr) => {
        // TODO: Fix ugly code
        // TODO: Make more efficient, this is currently a linear search
        if let Some((current_player, controller)) = (&mut player, &mut controller).iter().filter(|&(ref player, _)| player.address == addr).next(){
          current_player.connected = true;
          current_player.last_msg = delta.now;
          disabled.remove(controller.subject);
          // Dodging borrow checker, by returning instead of else-ing
          outbound.push(OutboundEvent::Directed{dest: addr, event: ServerNetworkEvent::Connected});
          return;
        } else {
          //self.new_player_addr.send(addr);
        }

        // Else: no existing entity
        let player_ent = arg.create();
        let object_ent = arg.create();
        player.insert(player_ent.clone(), PlayerAspect {
          address: addr,
          last_msg: delta.now,
          connected: true
        });

        controller.insert(player_ent.clone(), ControllerAspect {
          subject: object_ent.clone()
        });

        render.insert(object_ent.clone(), RenderAspect::new());
        physical.insert(object_ent.clone(), PhysicalAspect::new((0.0, 0.0, 5.0),(0.0, 0.0, 0.0), false));
        collision.insert(object_ent.clone(), CollisionAspect::new());
        outbound.push(OutboundEvent::Directed{dest: addr, event: ServerNetworkEvent::Connected})
      },
      &ConnectEvent::Disconnect(addr) => {
        println!("disconnect event");
        // TODO: Fix ugly code
        // TODO: Make more efficient, this is currently a linear search
        if let Some((current_player, controller)) = (&mut player, &mut controller).iter().filter(|&(ref player, _)| player.address == addr).next(){
          current_player.connected = false;
          disabled.insert(controller.subject, DisabledAspect::default());
          // Dodging borrow checker, by returning instead of else-ing
          outbound.push(OutboundEvent::Directed{dest: addr, event: ServerNetworkEvent::Disconnected});
          return;
        }
        // Else: no existing entity
        outbound.push(OutboundEvent::Directed{
          dest: addr,
          event: ServerNetworkEvent::Error("Tried to disconnect, but not connected to server".to_owned())
        });
      }
    });
  }
}
