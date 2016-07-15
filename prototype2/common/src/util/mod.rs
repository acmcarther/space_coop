pub trait Newness {
  fn is_newer_than(&self, other: &Self) -> bool;
}

impl Newness for u16 {
  // Defined as "is large than other by less than 1/2 u16::MAX, arithmetically
  // wrapped"
  fn is_newer_than(&self, other: &u16) -> bool {
    let pos_diff = self.wrapping_sub(*other);
    pos_diff != 0 && pos_diff < 32000
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn u16() {
    assert!(!0.is_newer_than(&0));
    assert!(1.is_newer_than(&0));
    assert!(30000.is_newer_than(&0));
    assert!(!33000.is_newer_than(&0));
  }
}
