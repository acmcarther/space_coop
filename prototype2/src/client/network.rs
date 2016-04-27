use std::net::SocketAddr;
use std::thread;
use std::time::Duration as StdDuration;
use std::str;
use std::io::Read;
use std::mem;

use gaffer_udp::GafferPacket;
use gaffer_udp::non_blocking::GafferSocket;
use itertools::Unfold;
use serde_json;
use flate2::read::GzDecoder;
use itertools::Itertools;

use common::util::Newness;
use common::world::ClientWorld;
use common::protocol::{StateFragment, ClientNetworkEvent, ServerPayload, ServerNetworkEvent};

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

pub enum FragmentBuffer {
  None,
  Partial { seq_num: u16, pieces: Vec<Option<Vec<u8>>> }
}

impl FragmentBuffer {
  pub fn new() -> FragmentBuffer {
    FragmentBuffer::None
  }

  pub fn integrate(&mut self, partial: StateFragment) {
    let mut replace_self = false;
    match self {
      &mut FragmentBuffer::None => replace_self = true,
      &mut FragmentBuffer::Partial { ref seq_num, ref mut pieces } => {
        if partial.seq_num.is_newer_than(&seq_num) {
          replace_self = true;
        } else if partial.seq_num == *seq_num {
          // TODO: This is very slightly unsafe, reflect in type
          // TODO: Additionally, this clone is unnecessary, include because
          //   I was too lazy to dodge the borrow checker
          pieces[partial.idx as usize] = Some(partial.payload.clone());
        }
      }
    }

    // Dodging borrow checker
    if replace_self { self.replace_self_with_partial(partial) }
  }

  fn replace_self_with_partial(&mut self, partial: StateFragment) {
    let mut pieces = vec![None; partial.count as usize];
    pieces[partial.idx as usize] = Some(partial.payload);

    // Assignment to self ref to change enum variant
    mem::swap(self, &mut FragmentBuffer::Partial { seq_num: partial.seq_num, pieces: pieces })
  }
}

pub trait Defragmentable: Sized {
  fn defragment(buffer: &FragmentBuffer) -> Option<Self>;
}

impl Defragmentable for ClientWorld {
  fn defragment(buffer: &FragmentBuffer) -> Option<Self> {
    match buffer {
      &FragmentBuffer::None => None,
      &FragmentBuffer::Partial { seq_num: _, ref pieces } => {
        // TODO: optimize this -- iterates twice
        if pieces.iter().all(|p| p.is_some()) {
          let mut full_buffer = Vec::new();
          pieces.iter().cloned().foreach(|p| full_buffer.append(&mut p.unwrap()));
          let bytes: &[u8] = full_buffer.as_ref();
          let mut string = String::new();
          GzDecoder::new(bytes)
            .and_then(|mut decoder| decoder.read_to_string(&mut string)).ok()
            .and_then(|_| serde_json::from_str(&string).ok())
        } else {
          None
        }
      }
    }
  }
}
