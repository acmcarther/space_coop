pub mod systems;

use automatic_system_installer::{AutoInstaller, Dag};
use network::AdapterSystem;
use specs;
use state::Delta;
use std::any::TypeId;
use time;
use world::ServerWorld;

pub struct Engine {
  pub planner: specs::Planner<Delta>,
}


impl Engine {
  pub fn dependency_dag() -> Dag<TypeId> {
    let mut auto_installer = AutoInstaller::new();
    systems::install_auto_systems(&mut auto_installer);
    auto_installer.take_dag()
  }

  pub fn new(port: u16) -> Engine {
    let mut world = ServerWorld::new().world;

    // Specially initialize the network adapter system
    let network_adapter_system = AdapterSystem::new(port, &mut world);

    // Automatic system installation
    let mut installer = AutoInstaller::with_world(world);

    installer.auto_install_instance(network_adapter_system);
    systems::install_auto_systems(&mut installer);

    let planner = installer.apply(5 /* threads, arbitrary */);

    Engine { planner: planner }
  }

  pub fn tick(&mut self, dt: &time::Duration) {
    self.planner.dispatch(Delta {
      dt: dt.clone(),
      now: time::now(),
    });
  }
}
