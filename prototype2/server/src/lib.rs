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
extern crate igd;

#[macro_use(declare_dependencies, standalone_installer_from_new)]
extern crate automatic_system_installer;

/// Manages main loop and coordination of application components
///
pub mod engine;

/// Manages game state
///
pub mod world;


use engine::Engine;
use std::thread;
use std::time::Duration as StdDuration;
use time::Duration;

pub fn start(port: u16, use_upnp: bool) {
  println!("Starting server on {}", port);

  if use_upnp {
    println!("Forwarding port {}", port);
    set_up_upnp(port);
  }

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

  if use_upnp {
    println!("Unforwarding port {}", port);
    tear_down_upnp(port);
  }
}

fn set_up_upnp(port: u16) {
  use std::net::SocketAddrV4;
  use std::net::Ipv4Addr;

  let gateway = igd::search_gateway().unwrap();
  let ip_to_bind = gateway.get_external_ip().unwrap();
  let ip_with_port = SocketAddrV4::new(ip_to_bind, port);

  gateway.add_port(igd::PortMappingProtocol::UDP,
                   port,
                   ip_with_port,
                   0, // lease_duration
                   "SpaceCoop Server");
}

fn tear_down_upnp(port: u16) {
  let gateway = igd::search_gateway().unwrap();
  gateway.remove_port(igd::PortMappingProtocol::UDP, port);
}
