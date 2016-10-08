extern crate gaffer_udp;
extern crate serde_json;
extern crate ecs;

use gaffer_udp::GafferPacket;
use std::str;

pub type Address = std::net::SocketAddr;

include!(concat!(env!("OUT_DIR"), "/lib.rs"));

// TODO: Unify ServerPayload and ClientPayload types

#[derive(Clone, Debug)]
pub struct ServerPayload {
  pub address: Address,
  pub event: ServerNetworkEvent,
}

impl ServerPayload {
  pub fn new(address: Address, event: ServerNetworkEvent) -> ServerPayload {
    ServerPayload {
      address: address,
      event: event,
    }
  }

  // TODO: Result for a more informative failure?
  pub fn from_gaffer_packet(pkt: GafferPacket) -> Option<ServerPayload> {
    let address = pkt.addr;
    let payload = pkt.payload;
    str::from_utf8(payload.as_ref())
      .ok()
      .map(|s| s.trim_right_matches('\0'))
      .and_then(|s| serde_json::from_str(s).ok())
      .map(|event| {
        ServerPayload {
          address: address,
          event: event,
        }
      })
  }
}

#[derive(Clone, Debug)]
pub struct ClientPayload {
  pub address: Address,
  pub event: ClientNetworkEvent,
}

impl ClientPayload {
  // TODO: Result for a more informative failure?
  pub fn from_gaffer_packet(pkt: GafferPacket) -> Option<ClientPayload> {
    let address = pkt.addr;
    let payload = pkt.payload;
    str::from_utf8(payload.as_ref())
      .ok()
      .map(|s| s.trim_right_matches('\0'))
      .and_then(|s| serde_json::from_str(s).ok())
      .map(|event| {
        ClientPayload {
          address: address,
          event: event,
        }
      })
  }
}
