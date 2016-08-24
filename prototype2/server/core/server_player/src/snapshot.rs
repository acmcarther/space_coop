use std::collections::{HashMap, HashSet};

use specs;

use std::net::SocketAddr;

use network::{Fragmentable, OutboundEvent};
use common::protocol::ServerNetworkEvent;
use common::aspects::{CommonWorld, DisabledAspect, PhysicalAspect, RenderAspect, SynchronizedAspect};
use aspects::{ControllerAspect, PlayerAspect};
use state::Delta;
use pubsub::{PubSubStore, SubscriberToken};


#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SnapshotAckEvent {
  address: SocketAddr,
  idx: u16,
}

impl SnapshotAckEvent {
  pub fn new(address: SocketAddr, idx: u16) -> SnapshotAckEvent {
    SnapshotAckEvent {
      address: address,
      idx: idx,
    }
  }
}

/**
 * Manages the broadcast of state snapshots, and the receipt of ack for those snapshots
 *
 * Input: SnapshotAckEvent, ClientState(PlayerAspect, PhysicalAspec, RenderAspect, DisabledAspect,
 * ControllerAspect
 */
pub struct System {
  snapshot_idx: u16,
  snapshot_ack_sub_token: SubscriberToken<SnapshotAckEvent>,
}

impl System {
  pub fn new(world: &mut specs::World) -> System {
    System {
      snapshot_idx: 0,
      snapshot_ack_sub_token: world.register_subscriber(),
    }
  }
}

#[allow(unused_variables, unused_imports)]
impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use specs::Join;
    use itertools::Itertools;

    self.snapshot_idx = self.snapshot_idx.wrapping_add(1);

    let (mut snapshot_ack_events,
         entities,
         synchronized,
         player,
         physical,
         render,
         disabled,
         controller,
         mut outbound_events) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.snapshot_ack_sub_token).collected(),
       w.entities(),
       w.read::<SynchronizedAspect>(),
       w.read::<PlayerAspect>(),
       w.read::<PhysicalAspect>(),
       w.read::<RenderAspect>(),
       w.read::<DisabledAspect>(),
       w.read::<ControllerAspect>(),
       w.fetch_publisher::<OutboundEvent>())
    });

    // TODO(acmcarther): Something useful with this event
    snapshot_ack_events.drain(..);

    let mut entity_set = HashSet::new();
    let mut physical_map = HashMap::new();
    let mut render_map = HashMap::new();
    let mut disabled_map = HashMap::new();

    // Get ent list
    (&entities, &synchronized).iter().foreach(|(_, synchro)| {
      // WARNING: Dropping generation, result may be invalid
      entity_set.insert(synchro.clone());
    });

    // Translated physical list
    (&entities, &synchronized, &physical).iter().foreach(|(entity, synchro, aspect)| {
      physical_map.insert(synchro.clone().to_string(), aspect.clone());
    });

    // Translated rendered list
    (&entities, &synchronized, &render).iter().foreach(|(entity, synchro, aspect)| {
      render_map.insert(synchro.clone().to_string(), aspect.clone());
    });

    // Translated disabled list
    (&entities, &synchronized, &disabled).iter().foreach(|(entity, synchro, aspect)| {
      disabled_map.insert(synchro.clone().to_string(), aspect.clone());
    });

    // Add outbound state snapshot events per player
    (&player, &entities)
      .iter()
      .filter(|&(ply, _)| ply.connected)
      .flat_map(|(ply, entity)| {
        let common_world = CommonWorld {
          own_entity: controller.get(entity).map(|v| v.subject.clone()),
          entities: entity_set.clone(),
          rendered: render_map.clone(),
          physical: physical_map.clone(),
          disabled: disabled_map.clone(),
        };

        common_world.fragment_to_events(self.snapshot_idx)
          .into_iter()
          .map(|partial| (ply.address.clone(), partial))
          .collect::<Vec<(SocketAddr, ServerNetworkEvent)>>()
          .into_iter()
      })
      .map(|(addr, event)| {
        OutboundEvent::Directed {
          dest: addr,
          event: event,
        }
      })
      .foreach(|e| outbound_events.push(e));
  }
}
