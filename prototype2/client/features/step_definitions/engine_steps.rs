use cucumber::CucumberRegistrar;

use state::Delta;
use specs;
use time;
use client::engine;
use support::{ClientWorld, Initializer};
use std::mem;
use std::ops::Deref;
use std::collections::HashMap;
use automatic_system_installer::{AutoInstaller, DependencyAware};
use itertools::Itertools;

lazy_static! {
  pub static ref SYSTEM_LOOKUP: HashMap<String, Box<Fn(&mut AutoInstaller<Delta>) + Sync>> = build_system_lookup();
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
    let mut auto_installer = AutoInstaller::<Delta>::new();
    let system_install_result = table.into_iter().foreach(|mut entry| {
      let item = entry.into_iter().nth(0).unwrap(); // TODO: this is hard error, make this more ergonomic
      SYSTEM_LOOKUP.get(&item)
        .map(|v| v(&mut auto_installer))
        .unwrap_or_else(|| panic!("Couldn't find system {}", item))
    });

    world.planner = auto_installer.apply(1);
  });
}
macro_rules! insert_system {
  ($map:ident, $system:ty) => {
    let f: Box<Fn(&mut AutoInstaller<Delta>) + Sync> = Box::new(move |auto_installer: &mut AutoInstaller<Delta>| {auto_installer.auto_install::<$system>(); });
    $map.insert(stringify!($system).to_owned(), f);
  }
}

fn build_system_lookup() -> HashMap<String, Box<Fn(&mut AutoInstaller<Delta>) + Sync>> {
  use client::*;
  use client::engine::*;
  // NOTE: Network adapter system omitted as it has side-effects
  // It doesn't make a lot of sense to test against it anyway
  let mut map = HashMap::new();
  insert_system!(map, network::EventDistributionSystem);
  insert_system!(map, network::ConnectionSystem);
  insert_system!(map, pause::System);
  insert_system!(map, player::PreprocessorSystem);
  insert_system!(map, camera::PreprocessorSystem);
  insert_system!(map, console::PreprocessorSystem);
  insert_system!(map, player::MoveSystem);
  insert_system!(map, camera::MovementSystem);
  insert_system!(map, console::InputSystem);
  insert_system!(map, console::InvokeSystem);
  insert_system!(map, mutator::System);
  insert_system!(map, synchronization::System);
  insert_system!(map, network::KeepAliveSystem);

  map
}
