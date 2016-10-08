use common::protocol::{Address, ServerNetworkEvent, ServerPayload};
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub enum OutboundEvent {
  Directed {
    dest: Address,
    event: ServerNetworkEvent,
  },
  Undirected(ServerNetworkEvent),
}

impl OutboundEvent {
  // TODO: Investigate a result here instead of just Vec for richer error handling
  pub fn to_server_payloads(self, all_addresses: &Vec<SocketAddr>) -> Vec<ServerPayload> {
    use self::OutboundEvent::*;
    match self {
      Directed { dest, event } => vec![ServerPayload::new(dest, event)],
      Undirected(event) => {
        all_addresses.iter().map(|a| ServerPayload::new(a.clone(), event.clone())).collect()
      },
    }
  }
}
