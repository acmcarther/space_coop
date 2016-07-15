pub mod physics;
pub mod io;
pub mod player;
pub mod debug;

use time;

use world::ServerWorld;

use specs;

const NETWORK_IO_PRIORITY: specs::Priority = 100;
const NETWORK_EVENT_DISTRIBUTION_PRIORITY: specs::Priority = 80;
const NETWORK_HEALTH_CHECK_PRIORITY: specs::Priority = 70;
const PHYSICS_PRIORITY: specs::Priority = 9;
const PLAYER_CONNECTION_PRIORITY: specs::Priority = 8;
const PLAYER_SNAPSHOT_PRIORITY: specs::Priority = 6;
const PLAYER_INPUT_PRIORITY: specs::Priority = 5;
const DEBUG_PRIORITY: specs::Priority = 1;

pub struct Engine {
  pub planner: specs::Planner<Delta>,
}

#[derive(Debug, Clone)]
pub struct Delta {
  pub dt: time::Duration,
  pub now: time::Tm,
}

impl Engine {
  //pub fn push_event(&mut self, event: ClientPayload) { self.events.push(event) }

  pub fn new(port: u16) -> Engine {

    let neo_world = ServerWorld::new();
    let mut planner = specs::Planner::new(neo_world.world, 2 /* Threads, arbitrary */);

    planner.add_system(io::network_adapter::System::new(port), "network::io", NETWORK_IO_PRIORITY);
    planner.add_system(io::event_distribution::System::new(), "network::event_distribution", NETWORK_EVENT_DISTRIBUTION_PRIORITY);
    planner.add_system(io::health_check::System::new(), "network::health_check", NETWORK_HEALTH_CHECK_PRIORITY);
    planner.add_system(physics::System::new(), "physics", PHYSICS_PRIORITY);
    planner.add_system(player::connection::System::new(), "player::connection", PLAYER_CONNECTION_PRIORITY);
    planner.add_system(player::snapshot::System::new(), "player::snapshot", PLAYER_SNAPSHOT_PRIORITY);
    planner.add_system(player::input::System::new(), "player::input", PLAYER_INPUT_PRIORITY);
    planner.add_system(debug::System::new(), "debug", DEBUG_PRIORITY);

    Engine {
      planner: planner
    }
  }

  pub fn tick(&mut self, dt: &time::Duration) {
    self.planner.dispatch(Delta {dt: dt.clone(), now: time::now()});
  }
}
