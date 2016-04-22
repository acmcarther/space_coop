use uuid::Uuid;

use common::network;

use common::protocol::{
  ServerNetworkEvent,
  ServerPayload
};

use std::net::SocketAddr;

#[derive(Clone)]
pub enum OutboundEvent {
  External { dest: network::Address, event: ServerNetworkEvent },
  Directed { dest: Uuid, event: ServerNetworkEvent },
  Undirected(ServerNetworkEvent)
}

impl OutboundEvent {
  // TODO: Investigate a result here instead of just Vec for richer error handling
  pub fn to_server_payloads(self, uuid_to_addr: &(Fn(Uuid) -> Option<SocketAddr>)) -> Vec<ServerPayload> {
    use self::OutboundEvent::*;
    match self {
      External {dest, event} => vec![ServerPayload::new(dest, event)],
      Directed { dest, event } => {
        match uuid_to_addr(dest) {
          Some(addr) => vec![ServerPayload::new(addr, event)],
          None => Vec::new()
        }
      }
      _ => Vec::new()
    }
  }
}
