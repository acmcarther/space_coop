use std::str;
use common::world::ClientWorld;
use serde_json;
use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct PartialSnapshot {
  pub series: u16,
  pieces: Vec<Option<Vec<u8>>>
}

impl PartialSnapshot {
  pub fn new(series: u16, idx: u32, count: u32, state_fragment: Vec<u8>) -> PartialSnapshot {
    let mut pieces = vec![None; count as usize];
    pieces[idx as usize] = Some(state_fragment);
    PartialSnapshot {
      series: series,
      pieces: pieces
    }
  }

  pub fn append(&mut self, idx: u32, state_fragment: Vec<u8>) {
    self.pieces[idx as usize] = Some(state_fragment);
  }

  pub fn other_series_newer(&self, series: u16) -> bool {
    let pos_diff = series.wrapping_sub(self.series);
    pos_diff != 0 && pos_diff < 32000
  }

  pub fn is_complete(&self) -> bool {
    self.pieces.iter().all(|p| p.is_some())
  }

  pub fn collate(&self) -> Option<ClientWorld> {
    let mut full_buffer = Vec::new();
    if self.pieces.iter().all(|p| p.is_some()) {
      // TODO: Take another look here, this is not efficient because of cloned
      self.pieces.iter().cloned().foreach(|mut p| full_buffer.append(&mut p.unwrap()));
      str::from_utf8(full_buffer.as_ref()).ok().and_then(|s| {
        println!("{}", s);
        serde_json::from_str(s).ok()
      })
    } else {
      println!("couldnt colate");
      None
    }
  }
}
