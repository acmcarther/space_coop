extern crate specs;
extern crate itertools;
extern crate flate2;
extern crate serde_json;
extern crate gaffer_udp;
extern crate time;

extern crate common;
extern crate server_state as state;
extern crate aspects;
extern crate pubsub;

mod protocol;
mod adapter;
mod fragmentation;

pub use protocol::OutboundEvent;
pub use adapter::System as AdapterSystem;

pub use self::fragmentation::Fragmentable;

use gaffer_udp::GafferPacket;
use gaffer_udp::non_blocking::GafferSocket;
use itertools::Unfold;

use common::protocol::{ClientPayload, ServerPayload};

/**
 * Manages the connection to the game clients
 *
 * Uses a GafferSocket (a wrapper around UDP for some reliability)
 */
pub struct Network {
  socket: GafferSocket,
}

impl Network {
  pub fn new(port: u16) -> Network {
    let sock = GafferSocket::bind(("0.0.0.0", port)).unwrap();
    Network { socket: sock }
  }

  pub fn recv_pending(&mut self) -> Vec<ClientPayload> {
    Unfold::new((), |_| self.socket.recv().ok().and_then(|v| v))
      .filter_map(|gaffer_packet| ClientPayload::from_gaffer_packet(gaffer_packet))
      .collect()
  }

  pub fn send(&mut self, payload: ServerPayload) {
    let json = serde_json::to_string(&payload.event).unwrap();
    let _ = self.socket.send(GafferPacket::new(payload.address, json.as_bytes().to_vec()));
  }
}
