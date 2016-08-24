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

/// Describes shared client/server aspects
///
pub mod aspects;

/// Manages Network IO
///
pub mod network;

/// Convenience wrappers for builtin types
///
pub mod util;

pub mod model;
