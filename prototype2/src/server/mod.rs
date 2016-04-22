pub mod engine;
pub mod network;
pub mod world;
pub mod protocol;

use time::{self, Duration};

use self::engine::Engine;
use self::network::Network;
use itertools::Itertools;

use std::thread;

use std::time::Duration as StdDuration;

// TODO: Tick rate
pub fn start(port: u16) {
  println!("Starting server on {}", port);
  let mut engine = Engine::new();
  let mut network = Network::new(port);
  let running = true;
  let tick_rate = 66;
  let time_step = 1.0 / (tick_rate as f32); //s

  let mut next_time = time::now();

  println!("Server Started!");
  while running {
    if time::now() > next_time {
      network.recv_pending().into_iter()
        .foreach(|client_payload| engine.push_event(client_payload));

      engine.tick().into_iter()
        .foreach(|server_payload| network.send(server_payload));

      next_time = next_time + Duration::milliseconds((time_step * 1000.0) as i64);
    } else {
      thread::sleep(StdDuration::from_millis(2))
    }
  }
}
