use std::str;
use serde_json;
use itertools::Itertools;
use std::mem;

use common::world::ClientWorld;
use common::util::Newness;
use common::protocol::PartialClientSnapshot;

pub enum ClientWorldBuffer {
  None,
  Partial { series: u16, pieces: Vec<Option<Vec<u8>>> }
}

impl ClientWorldBuffer {
  pub fn new() -> ClientWorldBuffer {
    ClientWorldBuffer::None
  }

  pub fn integrate(&mut self, partial: PartialClientSnapshot) {
    let mut replace_self = false;
    match self {
      &mut ClientWorldBuffer::None => replace_self = true,
      &mut ClientWorldBuffer::Partial { ref series, ref mut pieces } => {
        if partial.series.is_newer_than(&series) {
          replace_self = true;
        } else if partial.series == *series {
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
      &mut ClientWorldBuffer::Partial { ref mut series, ref mut pieces } => {
        // TODO: optimize this -- iterates twice
        if pieces.iter().all(|p| p.is_some()) {
          let mut full_buffer = Vec::new();
          pieces.iter().cloned().foreach(|mut p| full_buffer.append(&mut p.unwrap()));
          str::from_utf8(full_buffer.as_ref()).ok().and_then(|s| serde_json::from_str(s).ok())
        } else {
          None
        }
      }
    }
  }

  fn replace_self_with_partial(&mut self, partial: PartialClientSnapshot) {
    let mut pieces = vec![None; partial.count as usize];
    pieces[partial.idx as usize] = Some(partial.state_fragment);

    // Assignment to self ref to change enum variant
    mem::swap(self, &mut ClientWorldBuffer::Partial { series: partial.series, pieces: pieces })
  }
}
