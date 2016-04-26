use std::str;
use serde_json;
use itertools::Itertools;
use std::mem;

use common::world::ClientWorld;
use common::util::Newness;
use common::protocol::FullClientSnapshotFragment;

use flate2::read::GzDecoder;
use std::io::Read;

pub enum ClientWorldBuffer {
  None,
  Partial { seq_num: u16, pieces: Vec<Option<Vec<u8>>> }
}

impl ClientWorldBuffer {
  pub fn new() -> ClientWorldBuffer {
    ClientWorldBuffer::None
  }

  pub fn integrate(&mut self, partial: FullClientSnapshotFragment) {
    let mut replace_self = false;
    match self {
      &mut ClientWorldBuffer::None => replace_self = true,
      &mut ClientWorldBuffer::Partial { ref seq_num, ref mut pieces } => {
        if partial.seq_num.is_newer_than(&seq_num) {
          replace_self = true;
        } else if partial.seq_num == *seq_num {
          // TODO: This is very slightly unsafe, reflect in type
          // TODO: Additionally, this clone is unnecessary, include because
          //   I was too lazy to dodge the borrow checker
          pieces[partial.idx as usize] = Some(partial.state_fragment.clone());
        }
      }
    }

    // Dodging borrow checker
    if replace_self { self.replace_self_with_partial(partial) }
  }

  pub fn try_collate(&mut self) -> Option<ClientWorld> {
    match self {
      &mut ClientWorldBuffer::None => None,
      &mut ClientWorldBuffer::Partial { seq_num: _, ref mut pieces } => {
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

  fn replace_self_with_partial(&mut self, partial: FullClientSnapshotFragment) {
    let mut pieces = vec![None; partial.count as usize];
    pieces[partial.idx as usize] = Some(partial.state_fragment);

    // Assignment to self ref to change enum variant
    mem::swap(self, &mut ClientWorldBuffer::Partial { seq_num: partial.seq_num, pieces: pieces })
  }
}
