use std::net::SocketAddr;

use uuid::Uuid;

use common::network;
use common::protocol::{
  ServerNetworkEvent,
  ServerPayload
};

#[derive(Clone, Debug)]
pub enum OutboundEvent {
  External { dest: network::Address, event: ServerNetworkEvent },
  Directed { dest: Uuid, event: ServerNetworkEvent },
  Undirected(ServerNetworkEvent)
}

impl OutboundEvent {
  // TODO: Investigate a result here instead of just Vec for richer error handling
  pub fn to_server_payloads(self, all_addresses: &Vec<SocketAddr>, uuid_to_addr: &(Fn(Uuid) -> Option<SocketAddr>)) -> Vec<ServerPayload> {
    use self::OutboundEvent::*;
    match self {
      External {dest, event} => vec![ServerPayload::new(dest, event)],
      Directed {dest, event} => {
        match uuid_to_addr(dest) {
          Some(addr) => vec![ServerPayload::new(addr, event)],
          None => Vec::new()
        }
      }
      Undirected(event) => all_addresses.iter().map(|a| ServerPayload::new(a.clone(), event.clone())).collect()
    }
  }
}
