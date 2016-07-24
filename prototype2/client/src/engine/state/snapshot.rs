use specs;
use engine;

use common::protocol::{ClientNetworkEvent, SnapshotEvent};
use common::world::CommonWorld;
use network::FragmentBuffer;
use network::defragmentation::Defragmentable;
use itertools::Itertools;
use common::world::{DisabledAspect, PhysicalAspect, RenderAspect, SynchronizedAspect};
use specs::Join;
use world::OwnEntity;
use std::mem;
use std::collections::{HashMap, HashSet};

/**
 * Collects state snapshot messages from the server, collating them and integrating them into
 * current client state
 */
pub struct System {
  partial_snapshot: FragmentBuffer,
}

impl System {
  pub fn new() -> System {
    System { partial_snapshot: FragmentBuffer::None }
  }
}

impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    let (mut snapshot_events,
         mut outbound_events,
         mut own_entity,
         entities,
         mut synchronized,
         mut disabled,
         mut physical,
         mut rendered) = arg.fetch(|w| {
      (w.write_resource::<Vec<SnapshotEvent>>(),
       w.write_resource::<Vec<ClientNetworkEvent>>(),
       w.write_resource::<Option<OwnEntity>>(),
       w.entities(),
       w.write::<SynchronizedAspect>(),
       w.write::<DisabledAspect>(),
       w.write::<PhysicalAspect>(),
       w.write::<RenderAspect>())
    });

    let last_world = snapshot_events.drain(..)
      .filter_map(|event| {
        match event {
          SnapshotEvent::PartialSnapshot(state_fragment) => {
            self.partial_snapshot.integrate(state_fragment);
            let world_opt = CommonWorld::defragment(&self.partial_snapshot);
            if world_opt.is_some() {
              let (seq_num, world) = world_opt.unwrap();
              outbound_events.push(ClientNetworkEvent::SnapshotAck(seq_num));
              Some(world)
            } else {
              None
            }
          },
        }
      })
      .last();

    if let Some(mut world) = last_world {
      // Build synchro -> entity map and our set of synchros
      let mut synchro_to_entity = HashMap::new();
      let mut synchro_set = HashSet::new();
      (&entities, &synchronized).iter().foreach(|(ent, synchro)| {
        synchro_to_entity.insert(synchro.clone(), ent.clone());
        synchro_set.insert(synchro.clone());
      });

      // Delete entities omitted from the snapshot
      synchro_set.difference(&world.entities)
        .map(|synchro| synchro_to_entity.get(synchro).unwrap().clone())
        .foreach(|ent| arg.delete(ent.clone()));

      // Add new entities in snapshot to our ECS, and to synchro_to_entity mapping
      world.entities.difference(&synchro_set).into_iter().foreach(|synchro| {
        let ent = arg.create();
        synchronized.insert(ent.clone(), synchro.clone());
        synchro_to_entity.insert(synchro.clone(), ent);
      });

      // Set which entity is "us"
      *own_entity = world.own_entity.take().map(|e| OwnEntity(e));

      // Update our own entities from the shared synchros
      let mut entities = HashSet::new();
      mem::swap(&mut entities, &mut world.entities);
      entities.into_iter().foreach(|e| {
        // We know the ent is present because it either was already there, or got added.
        let our_ent = synchro_to_entity.get(&e).unwrap().clone();
        let rendered_in_world = world.rendered.remove(&e.to_string());
        let physical_in_world = world.physical.remove(&e.to_string());
        let disabled_in_world = world.disabled.remove(&e.to_string());

        if let Some(aspect) = rendered_in_world {
          rendered.insert(our_ent.clone(), aspect);
        } else {
          rendered.remove(our_ent.clone());
        }
        if let Some(aspect) = physical_in_world {
          physical.insert(our_ent.clone(), aspect);
        } else {
          physical.remove(our_ent.clone());
        }
        if let Some(aspect) = disabled_in_world {
          disabled.insert(our_ent.clone(), aspect);
        } else {
          disabled.remove(our_ent.clone());
        }
      });
    }
  }
}
