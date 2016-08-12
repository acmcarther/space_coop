extern crate itertools;
extern crate specs;
extern crate time;

extern crate common;
extern crate pubsub;
extern crate aspects;
extern crate server_state as state;
extern crate server_network as network;
extern crate server_player as player;
extern crate physics;

/// Manages main loop and coordination of application components
///
pub mod engine;

/// Manages game state
///
pub mod world;

use std::thread;
use std::time::Duration as StdDuration;

use time::Duration;

use engine::Engine;

pub fn start(port: u16) {
  println!("Starting server on {}", port);
  // TODO(acmcarther): Passing just 'port' to engine seems weird
  let mut engine = Engine::new(port);
  let running = true;
  let tick_rate = 66;
  let time_step = 1.0 / (tick_rate as f32); //s

  let mut last_time = time::now();
  let mut next_time = time::now();
  let mut now;

  println!("Server Started!");
  while running {
    now = time::now();
    if now > next_time {
      let dt = now - last_time;

      engine.tick(&dt);

      last_time = now;
      next_time = next_time + Duration::milliseconds((time_step * 1000.0) as i64);
    } else {
      thread::sleep(StdDuration::from_millis(2))
    }
  }
}
