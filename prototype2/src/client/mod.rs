pub mod renderer;
pub mod controller;
pub mod network;
pub mod engine;
pub mod world;
pub mod protocol;

use time::{self, Duration};

use common::protocol::ClientNetworkEvent;

use std::net::SocketAddr;

use self::engine::Engine;
use self::network::Network;
use itertools::Itertools;

use std::thread;

use std::time::Duration as StdDuration;

pub fn start(port: u16, server_addr: SocketAddr) {
  println!("Starting client on {}", port);
  let mut engine = Engine::new();
  let mut network = Network::new(port, server_addr);
  let running = true;
  let frame_limit = 60;
  let time_step = 1.0 / (frame_limit as f32); //s

  println!("Trying to connect");
  let success = network.connect();
  if !success { println!("Could not connect to {}", server_addr.to_string()); return }

  let mut next_time = time::now();
  let mut next_keepalive_time = time::now();

  println!("Client Started!");
  while running {
    if time::now() > next_time {
      network.recv_pending().into_iter()
        .foreach(|server_payload| engine.push_event(server_payload));

      if time::now() > next_keepalive_time {
        network.send(ClientNetworkEvent::KeepAlive);
        next_keepalive_time = next_keepalive_time + Duration::milliseconds(20);
      }

      engine.tick().into_iter()
        .foreach(|client_payload| network.send(client_payload));

      next_time = next_time + Duration::milliseconds((time_step * 1000.0) as i64);
    } else {
      thread::sleep(StdDuration::from_millis(2))
    }
  }

  println!("Client Disconnecting!");
  network.disconnect();
  println!("Client Disconnected!");
}
