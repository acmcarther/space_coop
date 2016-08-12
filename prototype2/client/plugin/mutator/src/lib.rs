extern crate specs;
extern crate itertools;
extern crate console;
extern crate pubsub;
extern crate state;
extern crate common;

use state::Delta;
use console::{Command, ConsoleLog};
use pubsub::{PubSubStore, Publisher, SubscriberToken};
use state::ExitFlag;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use common::world::{PhysicalAspect, RenderAspect, SynchronizedAspect};
use common::protocol::{ClientEvent, ClientNetworkEvent};
use common::model::ModelType;
use specs::Join;
use itertools::Itertools;

type AspectStorageRead<'a, T> = specs::Storage<T,
                                               RwLockReadGuard<'a, specs::Allocator>,
                                               RwLockReadGuard<'a, specs::MaskedStorage<T>>>;

type AspectStorageWrite<'a, T> = specs::Storage<T,
                                                RwLockReadGuard<'a, specs::Allocator>,
                                                RwLockWriteGuard<'a, specs::MaskedStorage<T>>>;

/**
 * Handle commands emitted by console
 */
pub struct System {
  commands_sub_token: SubscriberToken<Command>,
}

impl System {
  pub fn new(world: &mut specs::World) -> System {
    System { commands_sub_token: world.register_subscriber::<Command>() }
  }

  pub fn name() -> &'static str {
    "mutuator::System"
  }
}


impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use itertools::Itertools;

    let (mut commands,
         synchros,
         physicals,
         renders,
         mut client_events,
         mut exit_flag,
         mut console_log) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.commands_sub_token).collected(),
       w.read::<SynchronizedAspect>(),
       w.read::<PhysicalAspect>(),
       w.read::<RenderAspect>(),
       w.fetch_publisher::<ClientNetworkEvent>(),
       w.write_resource::<ExitFlag>(),
       w.write_resource::<ConsoleLog>())
    });

    commands.drain(..).foreach(|e| {
      match e {
        Command::Exit => self.exit(&mut exit_flag, &mut console_log),
        Command::ListEntities => self.list_entities(&synchros, &mut console_log),
        Command::ShowEntity(id) => {
          match synchro_from_id(&id, &synchros) {
            Some(synchro) => {
              self.show_entity(&synchro, &synchros, &physicals, &renders, &mut console_log)
            },
            None => console_log.push(format!("No entity starting with: {}", id)),
          }
        },
        Command::CreateEntity => {
          client_events.push(ClientNetworkEvent::DomainEvent(ClientEvent::CreateEntity));
          console_log.push("Creating entity...".to_owned());
        },
        Command::DeleteEntity(id) => {
          match synchro_from_id(&id, &synchros) {
            Some(synchro) => {
              client_events.push(ClientNetworkEvent::DomainEvent(ClientEvent::DeleteEntity(synchro.clone())));
              console_log.push(format!("Deleting entity {:?}", synchro));
            },
            None => console_log.push(format!("No entity starting with: {}", id)),
          }
        }
        Command::SetEntityPos(id, pos) => {
          match synchro_from_id(&id, &synchros) {
            Some(synchro) => {
              self.set_pos(&synchro, pos, &synchros, &physicals, &mut client_events, &mut console_log);
            },
            None => console_log.push(format!("No entity starting with: {}", id)),
          }
        }
        Command::SetEntityModel(id, model) => {
          match synchro_from_id(&id, &synchros) {
            Some(synchro) => {
              self.set_model(&synchro, model, &synchros, &renders, &mut client_events, &mut console_log);
            },
            None => console_log.push(format!("No entity starting with: {}", id)),
          }
        }
        Command::Help => self.help(&mut console_log),
      }
    });
  }
}

impl System {
  fn exit(&mut self, exit_flag: &mut ExitFlag, console_log: &mut ConsoleLog) {
    console_log.push("Exiting".to_owned());
    *exit_flag = ExitFlag(true);
  }

  fn help(&mut self, console_log: &mut ConsoleLog) {
    console_log.push(format!("Valid commands: {}", Command::print_all()));
  }

  fn list_entities<'a>(&mut self,
                       synchros: &AspectStorageRead<'a, SynchronizedAspect>,
                       console_log: &mut ConsoleLog) {

    synchros.iter().foreach(|synchro| console_log.push(format!("{:?}", synchro)));
  }

  // TODO: Optimize
  // This is way off the critical path, so i just hacked it up
  fn show_entity<'a>(&mut self,
                     synchro: &SynchronizedAspect,
                     synchros: &AspectStorageRead<'a, SynchronizedAspect>,
                     physicals: &AspectStorageRead<'a, PhysicalAspect>,
                     renderables: &AspectStorageRead<'a, RenderAspect>,
                     console_log: &mut ConsoleLog) {

    match (synchros, physicals, renderables)
      .iter()
      .find(|&(other_synchro, _, _)| *synchro == *other_synchro) {
      Some((_, physical, render)) => {
        console_log.push(format!("{:?}: {:?}, {:?}", synchro, physical, render))
      },
      None => {
        console_log.push(format!("Synchro {:?} is missing physical or rendered aspect",
                                 synchro))
      },
    }
  }


  // TODO: Optimize
  // This is way off the critical path, so i just hacked it up
  fn set_pos<'a>(&mut self,
                 synchro: &SynchronizedAspect,
                 pos: (f32, f32, f32),
                 synchros: &AspectStorageRead<'a, SynchronizedAspect>,
                 physicals: &AspectStorageRead<'a, PhysicalAspect>,
                 client_events: &mut Publisher<'a, ClientNetworkEvent>,
                 console_log: &mut ConsoleLog) {

    match (synchros, physicals)
      .iter()
      .find(|&(other_synchro, _)| *synchro == *other_synchro) {
      Some((_, physical)) => {
        console_log.push(format!("Setting pos of {:?}: to {:?}", &synchro, &pos));
        client_events.push(ClientNetworkEvent::DomainEvent(ClientEvent::MutatePhysicalAspect(synchro.clone(), physical.duplicate_with_pos(pos))));
      },
      None => console_log.push(format!("Synchro {:?} is missing physical aspect", synchro)),
    }
  }

  fn set_model<'a>(&mut self,
                   synchro: &SynchronizedAspect,
                   model: ModelType,
                   synchros: &AspectStorageRead<'a, SynchronizedAspect>,
                   renderables: &AspectStorageRead<'a, RenderAspect>,
                   client_events: &mut Publisher<'a, ClientNetworkEvent>,
                   console_log: &mut ConsoleLog) {
    match (synchros, renderables)
      .iter()
      .find(|&(other_synchro, _)| *synchro == *other_synchro) {
      Some((_, _)) => {
        console_log.push(format!("Setting model of {:?}: to {:?}", &synchro, &model));
        client_events.push(ClientNetworkEvent::DomainEvent(ClientEvent::MutateRenderAspect(synchro.clone(), RenderAspect::new_with(model))));
      },
      None => console_log.push(format!("Synchro {:?} is missing rendered aspect", synchro)),
    }
  }
}

fn synchro_from_id<'a>(id: &str,
                       synchros: &AspectStorageRead<'a, SynchronizedAspect>)
                       -> Option<SynchronizedAspect> {
  synchros.iter().find(|s| s.starts_with(&id)).cloned()
}
