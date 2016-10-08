extern crate itertools;
extern crate common;
extern crate time;

use common::ecs::aspects::SynchronizedAspect;

#[derive(Debug, Clone)]
pub struct ExitFlag(pub bool);

#[derive(Debug, Clone)]
pub struct OwnEntity(pub SynchronizedAspect);

#[derive(Debug, Clone)]
pub struct Delta {
  pub dt: time::Duration,
  pub now: time::Tm,
}
