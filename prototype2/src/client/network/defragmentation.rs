use std::io::Read;
use std::mem;

use serde_json;
use flate2::read::GzDecoder;
use itertools::Itertools;

use common::util::Newness;
use common::world::ClientWorld;
use common::protocol::StateFragment;

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
  fn defragment(buffer: &FragmentBuffer) -> Option<(u16, Self)>;
}

impl Defragmentable for ClientWorld {
  fn defragment(buffer: &FragmentBuffer) -> Option<(u16, Self)> {
    match buffer {
      &FragmentBuffer::None => None,
      &FragmentBuffer::Partial { seq_num, ref pieces } => {
        // TODO: optimize this -- iterates twice
        if pieces.iter().all(|p| p.is_some()) {
          let mut full_buffer = Vec::new();
          pieces.iter().cloned().foreach(|p| full_buffer.append(&mut p.unwrap()));
          let bytes: &[u8] = full_buffer.as_ref();
          let mut string = String::new();
          GzDecoder::new(bytes)
            .and_then(|mut decoder| decoder.read_to_string(&mut string)).ok()
            .and_then(|_| serde_json::from_str(&string).ok())
            .map(|res| (seq_num, res))
        } else {
          None
        }
      }
    }
  }
}
