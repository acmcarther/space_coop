use std::io::Read;
use std::mem;

use serde_json;
use flate2::read::GzDecoder;
use itertools::Itertools;

use common::util::Newness;
use common::world::CommonWorld;
use common::protocol::StateFragment;

/**
 * Represents a partial state snapshot as received from the server.
 * These are ordered via a seq_num field.
 *
 * TODO(acmcarther): Why is pieces a Vec<Option<...>>?
 */
pub enum FragmentBuffer {
  None,
  Partial {
    seq_num: u16,
    pieces: Vec<Option<Vec<u8>>>,
  },
}

impl FragmentBuffer {
  pub fn new() -> FragmentBuffer {
    FragmentBuffer::None
  }

  /**
   * Takes another StateFragment and tries to integrate it into the current partial
   * - If a fragment with a newer seq_num is received, the current partial is discarded
   * - If a fragment with an older seq_num is received, it is discarded
   * - If a fragment with the current seq_num is received, it is integrated into us
   */
  pub fn integrate(&mut self, partial: StateFragment) {
    let mut replace_self = false;
    match self {
      &mut FragmentBuffer::None => replace_self = true,
      &mut FragmentBuffer::Partial { ref seq_num, ref mut pieces } => {
        if partial.seq_num.is_newer_than(&seq_num) {
          // Drop the previous buffer and use the new one
          replace_self = true;
        } else if partial.seq_num == *seq_num {
          // TODO: This is very slightly unsafe, reflect in type
          // TODO: Additionally, this clone is unnecessary, include because
          //   I was too lazy to dodge the borrow checker
          pieces[partial.idx as usize] = Some(partial.payload.clone());
        }
      },
    }

    // Dodging borrow checker
    // This allows self to be switched to a different variant without borrow
    // checker complaining
    if replace_self {
      self.replace_self_with_partial(partial)
    }
  }

  fn replace_self_with_partial(&mut self, partial: StateFragment) {
    let mut pieces = vec![None; partial.count as usize];
    pieces[partial.idx as usize] = Some(partial.payload);

    // Assignment to self ref to change enum variant
    mem::swap(self,
              &mut FragmentBuffer::Partial {
                seq_num: partial.seq_num,
                pieces: pieces,
              })
  }
}

/**
 * Indicates that the implementor can be constructed from FragmentBuffer partials
 *
 * Implementors:
 * - CommonWorld is received over the wire in this manner
 */
pub trait Defragmentable: Sized {
  /**
   * Builds a full object from the FragmentBuffer, returning the sequence number and the object
   */
  fn defragment(buffer: &FragmentBuffer) -> Option<(u16, Self)>;
}

impl Defragmentable for CommonWorld {
  /**
   * CommonWorld is Gzipped Json. Besides the common work of combining the partial bytes,
   * the payload must also be un-gzipped, and then parsed into a CommonWorld
   */
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
            .and_then(|mut decoder| decoder.read_to_string(&mut string))
            .ok()
            .and_then(|_| serde_json::from_str(&string).ok())
            .map(|res| (seq_num, res))
        } else {
          None
        }
      },
    }
  }
}
