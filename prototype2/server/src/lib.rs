extern crate uuid;
extern crate time;
extern crate serde;
extern crate serde_json;
extern crate itertools;
extern crate gaffer_udp;
extern crate flate2;
extern crate cgmath;

extern crate common;

/**
 * Manages main loop and coordination of application components
 */
pub mod engine;

/**
 * Manages network IO
 */
pub mod network;

/**
 * Manages game state
 */
pub mod world;

/**
 * Describes server outbound payloads
 */
pub mod protocol;

use std::thread;
use std::time::Duration as StdDuration;

use time::Duration;
use itertools::Itertools;

use engine::Engine;
use network::Network;

pub fn start(port: u16) {
  println!("Starting server on {}", port);
  let mut engine = Engine::new();
  let mut network = Network::new(port);
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
      network.recv_pending().into_iter()
        .foreach(|client_payload| engine.push_event(client_payload));

      engine.tick(&dt).into_iter()
        .foreach(|server_payload| network.send(server_payload));

      last_time = now;
      next_time = next_time + Duration::milliseconds((time_step * 1000.0) as i64);
    } else {
      thread::sleep(StdDuration::from_millis(2))
    }
  }
}
