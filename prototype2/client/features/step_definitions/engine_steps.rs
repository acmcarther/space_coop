use cucumber::CucumberRegistrar;

use state::Delta;
use specs;
use time;
use client::engine;
use support::{ClientWorld, Initializer};
use std::mem;
use std::ops::Deref;
use std::collections::HashMap;
use itertools::Itertools;

lazy_static! {
  pub static ref SYSTEM_LOOKUP: HashMap<String, Box<Fn(&mut specs::World, &mut Initializer) + Sync>> = build_system_lookup();
}

pub fn register_steps(c: &mut CucumberRegistrar<ClientWorld>) {
  When!(c,
        "^the engine runs once$",
        |_, world: &mut ClientWorld, _| {
    // Invocation of this step implies that the delta is unused
    world.planner.dispatch(Delta {
      dt: time::Duration::seconds(1),
      now: time::now(),
    });
  });

  Given!(c,
         "^an engine with:$",
         |_, world: &mut ClientWorld, (table,): (Vec<Vec<String>>,)| {
    let mut initializer = Initializer::new();
    let mut new_world = specs::World::new();
    let system_install_result = table.into_iter().foreach(|mut entry| {
      let item = entry.into_iter().nth(0).unwrap(); // TODO: this is hard error, make this more ergonomic
      SYSTEM_LOOKUP.get(&item)
        .map(|v| {
          v(&mut new_world, &mut initializer);
        })
        .unwrap_or_else(|| panic!("Couldn't find system {}", item))
    });

    world.planner = initializer.build(new_world);
  });
}
macro_rules! insert_system {
  ($map:ident, $priority:ident, $system:ty) => {
    let f: Box<Fn(&mut specs::World, &mut Initializer) + Sync> = Box::new(move |world: &mut specs::World, init: &mut Initializer| {
      init.add_system(<$system>::new(world), stringify!($system).to_owned(), $priority);
    });
    $map.insert(stringify!($system).to_owned(), f)
  }
}

fn build_system_lookup() -> HashMap<String, Box<Fn(&mut specs::World, &mut Initializer) + Sync>> {
  use client::*;
  use client::engine::*;
  // NOTE: Network adapter system omitted as it has side-effects
  // It doesn't make a lot of sense to test against it anyway
  let mut map = HashMap::new();
  insert_system!(map,
                 NETWORK_EVENT_DISTRIBUTION_PRIORITY,
                 network::EventDistributionSystem);
  insert_system!(map, NETWORK_CONNECTION_PRIORITY, network::ConnectionSystem);
  insert_system!(map, PAUSE_PRIORITY, pause::System);
  insert_system!(map,
                 PLAYER_PREPROCESSOR_PRIORITY,
                 player::PreprocessorSystem);
  insert_system!(map,
                 CAMERA_PREPROCESSOR_PRIORITY,
                 camera::PreprocessorSystem);
  insert_system!(map,
                 CONSOLE_PREPROCESSOR_PRIORITY,
                 console::PreprocessorSystem);
  insert_system!(map, PLAYER_MOVE_PRIORITY, player::MoveSystem);
  insert_system!(map, CAMERA_MOVE_PRIORITY, camera::MovementSystem);
  insert_system!(map, CONSOLE_INPUT_PRIORITY, console::InputSystem);
  insert_system!(map, CONSOLE_INVOKER_PRIORITY, console::InvokeSystem);
  insert_system!(map, MUTATOR_PRIORITY, mutator::System);
  insert_system!(map, STATE_SNAPSHOT_PRIORITY, synchronization::System);
  insert_system!(map, NETWORK_KEEP_ALIVE_PRIORITY, network::KeepAliveSystem);

  map
}
