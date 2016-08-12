extern crate time;

#[derive(Debug, Clone)]
pub struct Delta {
  pub dt: time::Duration,
  pub now: time::Tm,
}
