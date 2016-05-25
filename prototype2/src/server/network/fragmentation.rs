use std::io::Write;

use serde_json;
use flate2::write::GzEncoder;
use flate2::Compression;

use common::protocol::{
  ServerNetworkEvent,
  SnapshotEvent,
  StateFragment
};
use common::world::ClientWorld;

pub trait Fragmentable {
  fn fragment_to_events(&self, seq_num: u16) -> Vec<ServerNetworkEvent>;
}

impl Fragmentable for ClientWorld {
  fn fragment_to_events(&self, seq_num: u16) -> Vec<ServerNetworkEvent> {
    let client_snapshot = serde_json::to_string(&self).unwrap();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::Default);
    encoder.write(client_snapshot.as_bytes()).unwrap();
    let snapshot_bytes = encoder.finish().unwrap(); // Assumed to be safe because I control the format
    let snapshot_byte_sets = snapshot_bytes.chunks(128 /*bytes*/).enumerate();
    let set_count = snapshot_byte_sets.len();

    snapshot_byte_sets.map(|(idx, bytes)| {
      ServerNetworkEvent::Snapshot(SnapshotEvent::PartialSnapshot(StateFragment {
        seq_num: seq_num,
        idx: idx as u32,
        count: set_count as u32,
        payload: bytes.to_vec()
      }))
    }).collect::<Vec<ServerNetworkEvent>>()
  }
}
