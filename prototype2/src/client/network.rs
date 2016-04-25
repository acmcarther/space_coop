use gaffer_udp::GafferPacket;
use gaffer_udp::non_blocking::GafferSocket;
use itertools::Unfold;
use serde_json;

use std::net::SocketAddr;
use std::thread;
use std::time::Duration as StdDuration;

use common::protocol::{ClientNetworkEvent, ServerPayload, ServerNetworkEvent};

pub struct Network {
  socket: GafferSocket,
  port: u16,
  server_addr: SocketAddr
}

impl Network {
  pub fn new(port: u16, server_addr: SocketAddr) -> Network {
    let sock = GafferSocket::bind(("0.0.0.0", port)).unwrap();
    Network {
      socket: sock,
      port: port,
      server_addr: server_addr
    }
  }

  pub fn recv_pending(&mut self) -> Vec<ServerNetworkEvent> {
    let server_addr = self.server_addr.clone();
    Unfold::new((), |_| self.socket.recv().ok().and_then(|v| v))
      .filter_map(|gaffer_packet| ServerPayload::from_gaffer_packet(gaffer_packet))
      .filter(|payload| payload.address == server_addr)
      .map(|payload| payload.event)
      .collect()
  }

  pub fn send(&mut self, payload: ClientNetworkEvent) {
    let json = serde_json::to_string(&payload).unwrap();
    self.socket.send(GafferPacket::new(self.server_addr.clone(), json.as_bytes().to_vec()));
  }

  pub fn connect(&mut self) -> bool {
    self.try_send(ClientNetworkEvent::Connect, ServerNetworkEvent::Connected, 5)
  }
  pub fn disconnect(&mut self) -> bool {
    self.try_send(ClientNetworkEvent::Disconnect, ServerNetworkEvent::Disconnected, 5)
  }

  fn try_send(&mut self, event: ClientNetworkEvent, expected_event: ServerNetworkEvent, tries: u32) -> bool {
    let mut tries_remaining = tries;
    self.send(event.clone());
    thread::sleep(StdDuration::from_millis(200));

    while tries_remaining > 0 {
      let success = self.recv_pending().into_iter().any(|payload| payload == expected_event);
      if success {return true}
      tries_remaining = tries_remaining - 1;
      self.send(event.clone());
      thread::sleep(StdDuration::from_millis(200));
    }

    false
  }
}