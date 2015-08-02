pub use self::app_net::{
  ClientNet,
  ServerNet,
};

pub mod helpers;
mod serialize;
mod deserialize;

mod app_net {
  use std::net::SocketAddr;
  use std::sync::mpsc::SendError;

  use game_udp;
  use game_udp::types as game_udp_types;
  use game_udp::packet_types::Packet;
  use helpers::try_recv_all;
  use events::{ClientEvent, ServerEvent};

  use app_net::{serialize, deserialize};

  pub struct ClientNet {
    server_addr: SocketAddr,
    udp_net: game_udp::types::Network,
  }

  impl ClientNet {
    pub fn new(addr: SocketAddr, server_addr: SocketAddr) -> ClientNet {
      ClientNet {udp_net: game_udp::start_network(addr), server_addr: server_addr}
    }

    pub fn send_event(&self, event: ClientEvent) -> Option<SendError<Packet>> {
      let message = serialize::client_event(event);
      let packet = Packet { addr: self.server_addr.clone(), bytes: message };
      self.udp_net.send_channel.send(packet).err()
    }

    pub fn get_events(&self) -> Vec<ServerEvent> {
      try_recv_all(&self.udp_net.recv_channel)
        .into_iter()
        .filter(|packet| packet.addr == self.server_addr)
        .map(|packet| deserialize::server_event(packet.bytes))
        .filter(Option::is_some)
        .map(Option::unwrap)
        .collect()
    }
  }

  pub struct ServerNet {
    udp_net: game_udp_types::Network,
  }

  impl ServerNet {
    pub fn new(addr: SocketAddr) -> ServerNet {
      ServerNet {udp_net: game_udp::start_network(addr)}
    }

    pub fn send_event(&self, target: SocketAddr, event: ServerEvent) -> Option<SendError<Packet>> {
      let message = serialize::server_event(event);
      let packet = Packet { addr: target, bytes: message };
      self.udp_net.send_channel.send(packet).err()
    }

    pub fn get_events(&self) -> Vec<(SocketAddr, ClientEvent)> {
      try_recv_all(&self.udp_net.recv_channel)
        .into_iter()
        .map(|packet| {
          let addr = packet.addr;
          deserialize::client_event(packet.bytes).map(|event| (addr, event))
        })
        .filter(Option::is_some)
        .map(Option::unwrap)
        .collect()
    }
  }
}
