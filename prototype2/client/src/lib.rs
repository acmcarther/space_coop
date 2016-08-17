extern crate time;
extern crate specs;
extern crate itertools;
#[macro_use]
extern crate gfx;
extern crate glutin;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate common;
extern crate pubsub;

pub extern crate client_network as network;
pub extern crate camera;
pub extern crate console;
pub extern crate renderer;
pub extern crate debug;
pub extern crate synchronization;
pub extern crate mouse_lock;
pub extern crate window;
pub extern crate client_player as player;
pub extern crate pause;
pub extern crate mutator;
pub extern crate client_state as state;

pub mod engine;
pub mod world;

use std::thread;
use std::net::SocketAddr;
use std::time::Duration as StdDuration;

use time::Duration;

use engine::Engine;

/**
 * A function to begin running the client
 */
pub fn start(port: u16, server_addr: SocketAddr) {
  println!("Starting client on {}", port);
  let mut engine = Engine::new(port, server_addr.clone());
  let frame_limit = 60;
  let time_step = 1.0 / (frame_limit as f32); //s

  let mut last_time = time::now();
  let mut next_time = time::now();
  let mut now;

  println!("Client Started!");
  while engine.running() {
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

  // Finish up
  let dt = time::now() - last_time;
  engine.finalize(&dt);
}
