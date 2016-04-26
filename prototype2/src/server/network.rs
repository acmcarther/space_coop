use std::io::Write;

use gaffer_udp::GafferPacket;
use gaffer_udp::non_blocking::GafferSocket;
use itertools::Unfold;
use serde_json;
use flate2::write::GzEncoder;
use flate2::Compression;

use common::protocol::{
  ClientPayload,
  ServerPayload,
  ServerNetworkEvent,
  SnapshotEvent,
  FullClientSnapshotFragment
};
use common::world::{ClientWorld};

pub struct Network {
  socket: GafferSocket,
  port: u16
}

impl Network {
  pub fn new(port: u16) -> Network {
    let sock = GafferSocket::bind(("0.0.0.0", port)).unwrap();
    Network { socket: sock, port: port }
  }

  pub fn recv_pending(&mut self) -> Vec<ClientPayload> {
    Unfold::new((), |_| self.socket.recv().ok().and_then(|v| v))
      .filter_map(|gaffer_packet| ClientPayload::from_gaffer_packet(gaffer_packet))
      .collect()
  }

  pub fn send(&mut self, payload: ServerPayload) {
    let json = serde_json::to_string(&payload.event).unwrap();
    self.socket.send(GafferPacket::new(payload.address, json.as_bytes().to_vec()));
  }
}

pub trait Fragmentable {
  fn fragment_to_events(&self, seq_num: u16) -> Vec<ServerNetworkEvent>;
}

impl Fragmentable for ClientWorld {
  fn fragment_to_events(&self, seq_num: u16) -> Vec<ServerNetworkEvent> {
    let client_snapshot = serde_json::to_string(&self).unwrap();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::Default);
    encoder.write(client_snapshot.as_bytes());
    let snapshot_bytes = encoder.finish().unwrap(); // Assumed to be safe because I control the format
    let snapshot_byte_sets = snapshot_bytes.chunks(128 /*bytes*/).enumerate();
    let set_count = snapshot_byte_sets.len();

    snapshot_byte_sets.map(|(idx, bytes)| {
      ServerNetworkEvent::Snapshot(SnapshotEvent::PartialSnapshot(FullClientSnapshotFragment {
        seq_num: seq_num,
        idx: idx as u32,
        count: set_count as u32,
        state_fragment: bytes.to_vec()
      }))
    }).collect::<Vec<ServerNetworkEvent>>()
  }
}
