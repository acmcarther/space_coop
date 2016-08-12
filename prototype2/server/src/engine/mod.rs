pub mod io;

use time;

use world::ServerWorld;

use specs;

use state::Delta;
use physics::System as PhysicsSystem;
use network::AdapterSystem;
use player::{ConnectionSystem, HealthCheckSystem, InputSystem, SnapshotSystem};

const NETWORK_IO_PRIORITY: specs::Priority = 100;
const NETWORK_EVENT_DISTRIBUTION_PRIORITY: specs::Priority = 80;
const NETWORK_HEALTH_CHECK_PRIORITY: specs::Priority = 70;
const PHYSICS_PRIORITY: specs::Priority = 9;
const PLAYER_CONNECTION_PRIORITY: specs::Priority = 8;
const PLAYER_SNAPSHOT_PRIORITY: specs::Priority = 6;
const PLAYER_INPUT_PRIORITY: specs::Priority = 5;

pub struct Engine {
  pub planner: specs::Planner<Delta>,
}


impl Engine {
  pub fn new(port: u16) -> Engine {
    let mut world = ServerWorld::new().world;

    let network_adapter_system = AdapterSystem::new(port, &mut world);
    let event_distribution_system = io::event_distribution::System::new(&mut world);
    let health_check_system = HealthCheckSystem::new(&mut world);
    let physics_system = PhysicsSystem::new(&mut world);
    let connection_system = ConnectionSystem::new(&mut world);
    let snapshot_system = SnapshotSystem::new(&mut world);
    let player_input_system = InputSystem::new(&mut world);

    let mut planner = specs::Planner::new(world, 2 /* Threads, arbitrary */);
    planner.add_system(network_adapter_system, "network::io", NETWORK_IO_PRIORITY);
    planner.add_system(event_distribution_system,
                       "network::event_distribution",
                       NETWORK_EVENT_DISTRIBUTION_PRIORITY);
    planner.add_system(health_check_system,
                       "network::health_check",
                       NETWORK_HEALTH_CHECK_PRIORITY);
    planner.add_system(physics_system, "physics", PHYSICS_PRIORITY);
    planner.add_system(connection_system,
                       "player::connection",
                       PLAYER_CONNECTION_PRIORITY);
    planner.add_system(snapshot_system,
                       "player::snapshot",
                       PLAYER_SNAPSHOT_PRIORITY);
    planner.add_system(player_input_system, "player::input", PLAYER_INPUT_PRIORITY);

    Engine { planner: planner }
  }

  pub fn tick(&mut self, dt: &time::Duration) {
    self.planner.dispatch(Delta {
      dt: dt.clone(),
      now: time::now(),
    });
  }
}
