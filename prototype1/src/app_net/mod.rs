pub use self::app_net::{
  ClientNet,
  ServerNet
};


mod app_net {
  use std::net::SocketAddr;
  use std::sync::mpsc::Receiver;
  use std::thread::JoinHandle;

  use game_udp;
  use helpers::try_recv_all;
  use events::{ClientEvent, ServerEvent};
  use state::{ClientState, ServerState};

  pub struct ClientNet {
    udp_net: game_udp::types::Network,
  }

  impl ClientNet {
    pub fn new(addr: SocketAddr) -> ClientNet {
      ClientNet {udp_net: game_udp::start_network(addr)}
    }

    pub fn send_event(&self, event: ClientEvent) {
    }

    pub fn integrate(&self, state: &mut ClientState) {
      try_recv_all(&self.udp_net.recv_channel).into_iter();
    }
  }

  pub struct ServerNet {
    udp_net: game_udp::types::Network,
  }

  impl ServerNet {
    pub fn new(addr: SocketAddr) -> ServerNet {
      ServerNet {udp_net: game_udp::start_network(addr)}
    }

    pub fn send_event(&self, event: ServerEvent) {
    }

    pub fn integrate(&self, state: &mut ServerState) {
      try_recv_all(&self.udp_net.recv_channel).into_iter();
    }
  }
}
