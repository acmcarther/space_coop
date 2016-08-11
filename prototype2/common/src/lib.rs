extern crate serde;
extern crate serde_json;
extern crate gaffer_udp;
extern crate specs;
extern crate itertools;
extern crate uuid;
extern crate time;

/// Describes outbound and inbound payloads
///
pub mod protocol;

/// Describes all game state
///
pub mod world;

/// Manages Network IO
///
pub mod network;

/// Convenience wrappers for builtin types
///
pub mod util;

pub mod model;

#[derive(Debug, Clone)]
pub struct Delta {
  pub dt: time::Duration,
  pub now: time::Tm,
}
