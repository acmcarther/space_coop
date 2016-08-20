extern crate specs;
extern crate itertools;

extern crate client_network as network;
extern crate common;
extern crate pubsub;
extern crate client_state as state;
#[macro_use(declare_dependencies, standalone_installer_from_new)]
extern crate automatic_system_installer;

use state::{Delta, OwnEntity};
use common::protocol::{ClientNetworkEvent, SnapshotEvent};
use common::world::CommonWorld;
use network::FragmentBuffer;
use network::Defragmentable;
use itertools::Itertools;
use common::world::{DisabledAspect, PhysicalAspect, RenderAspect, SynchronizedAspect};
use std::mem;
use std::collections::{HashMap, HashSet};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use pubsub::{PubSubStore, Publisher, SubscriberToken};

type AspectStorageRead<'a, T> = specs::Storage<T,
                                               RwLockReadGuard<'a, specs::Allocator>,
                                               RwLockReadGuard<'a, specs::MaskedStorage<T>>>;

type AspectStorageWrite<'a, T> = specs::Storage<T,
                                                RwLockReadGuard<'a, specs::Allocator>,
                                                RwLockWriteGuard<'a, specs::MaskedStorage<T>>>;
/**
 * Collects state snapshot messages from the server, collating them and integrating them into
 * current client state
 */
pub struct System {
  partial_snapshot: FragmentBuffer,
  snapshot_event_sub_token: SubscriberToken<SnapshotEvent>,
}
declare_dependencies!(System, [network::EventDistributionSystem]);
standalone_installer_from_new!(System, Delta);

impl System {
  pub fn new(world: &mut specs::World) -> System {
    System {
      partial_snapshot: FragmentBuffer::None,
      snapshot_event_sub_token: world.register_subscriber::<SnapshotEvent>(),
    }
  }

  pub fn name() -> &'static str {
    "synchronization::System"
  }

  fn process_snapshots(&mut self,
                       snapshot_events: &mut Vec<SnapshotEvent>,
                       outbound_events: &mut Publisher<ClientNetworkEvent>)
                       -> Option<CommonWorld> {
    // Order the events by seq_num and idx (so we dont drop messages when we get
    // them out of order)
    snapshot_events.sort();

    snapshot_events.drain(..)
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
      .last()
  }

  fn incorporate_recvd_synchros(&mut self,
                                arg: &specs::RunArg,
                                world: &mut CommonWorld,
                                entities: &specs::Entities,
                                mut synchronized: AspectStorageWrite<SynchronizedAspect>)
                                -> HashMap<SynchronizedAspect, specs::Entity> {
    use specs::Join;

    let mut synchro_to_entity = HashMap::new();
    let mut synchro_set = HashSet::new();

    // Add our own synchros to our caches
    {
      (entities, &synchronized).iter().foreach(|(ent, synchro)| {
        synchro_to_entity.insert(synchro.clone(), ent.clone());
        synchro_set.insert(synchro.clone());
      });
    }

    // Delete entities omitted from the world snapshot
    synchro_set.difference(&world.entities)
      .map(|synchro| synchro_to_entity.get(synchro).unwrap().clone())
      .foreach(|ent| arg.delete(ent.clone()));

    // Add new entities in snapshot to our ECS, and to synchro_to_entity mapping
    world.entities.difference(&synchro_set).into_iter().foreach(|synchro| {
      let ent = arg.create();
      synchronized.insert(ent.clone(), synchro.clone());
      synchro_to_entity.insert(synchro.clone(), ent);
    });

    synchro_to_entity
  }

  fn update_entity(&mut self,
                   entity: specs::Entity,
                   synchro: SynchronizedAspect,
                   world: &mut CommonWorld,
                   rendered: &mut AspectStorageWrite<RenderAspect>,
                   physical: &mut AspectStorageWrite<PhysicalAspect>,
                   disabled: &mut AspectStorageWrite<DisabledAspect>) {
    let rendered_in_world = world.rendered.remove(&synchro.to_string());
    let physical_in_world = world.physical.remove(&synchro.to_string());
    let disabled_in_world = world.disabled.remove(&synchro.to_string());

    if let Some(aspect) = rendered_in_world {
      rendered.insert(entity.clone(), aspect);
    } else {
      rendered.remove(entity.clone());
    }
    if let Some(aspect) = physical_in_world {
      physical.insert(entity.clone(), aspect);
    } else {
      physical.remove(entity.clone());
    }
    if let Some(aspect) = disabled_in_world {
      disabled.insert(entity.clone(), aspect);
    } else {
      disabled.remove(entity.clone());
    }
  }
}

impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    let (mut snapshot_events,
         mut outbound_events,
         mut own_entity,
         entities,
         synchronized,
         mut disabled,
         mut physical,
         mut rendered) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.snapshot_event_sub_token).collected(),
       w.fetch_publisher::<ClientNetworkEvent>(),
       w.write_resource::<Option<OwnEntity>>(),
       w.entities(),
       w.write::<SynchronizedAspect>(),
       w.write::<DisabledAspect>(),
       w.write::<PhysicalAspect>(),
       w.write::<RenderAspect>())
    });

    let last_world = self.process_snapshots(&mut snapshot_events, &mut outbound_events);

    if let Some(mut world) = last_world {

      // Create new entities, remove old entities, and find the mapping from Synchro
      // -> Entity
      let synchro_to_entity =
        self.incorporate_recvd_synchros(&arg, &mut world, &entities, synchronized);

      // Set which entity is "us"
      *own_entity = world.own_entity.take().map(|e| OwnEntity(e));

      // Take the entities from the world (borrow checker trick for iterating)
      let mut entities = HashSet::new();
      mem::swap(&mut entities, &mut world.entities);

      // Update our own entities from the shared synchros
      entities.into_iter().foreach(|synchro| {
        // We know the ent is present because it either was already there, or got added.
        let client_ent = synchro_to_entity.get(&synchro).unwrap().clone();

        // Update our version of the entity from the world
        self.update_entity(client_ent,
                           synchro,
                           &mut world,
                           &mut rendered,
                           &mut physical,
                           &mut disabled);
      });
    }
  }
}
