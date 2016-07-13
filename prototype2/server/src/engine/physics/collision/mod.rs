mod detection;
mod refining;

use cgmath::Vector3;
use uuid::Uuid;
use std::collections::HashMap;

use common::world::PhysicalAspect;
use world::{CollisionAspect, ServerWorld};

pub use self::detection::{BroadPhaseDetector, BruteForceDetector};
pub use self::refining::{NarrowPhaseRefiner, BruteForceRefiner};

pub struct CollisionWorldView<'a> {
  pub physical: &'a HashMap<Uuid, PhysicalAspect>,
  pub collidable: &'a HashMap<Uuid, CollisionAspect>
}

impl <'a> CollisionWorldView<'a> {
  pub fn from_server_world(world: &'a ServerWorld) -> CollisionWorldView<'a> {
    CollisionWorldView {
      physical: &world.physical,
      collidable: &world.collidable
    }
  }
}

#[derive(PartialEq, Clone, Debug)]
pub struct DetectedCollision {
  pub entities: (Uuid, Uuid),
}

#[derive(PartialEq, Clone, Debug)]
pub struct CollisionDetail {
  pub entities: (Uuid, Uuid),
  pub interpenetration_resolution: (Vector3<f32>, Vector3<f32>),
  pub impulses: (Vector3<f32>, Vector3<f32>),
  pub torques: ((Vector3<f32>, f32), (Vector3<f32>, f32)),
}

pub trait CollisionDetector {
  fn detect(&mut self, world: &CollisionWorldView) -> Vec<CollisionDetail>;
}

#[allow(dead_code)]
pub struct NoCollisions;

impl NoCollisions {
  #[allow(dead_code)]
  pub fn new() -> NoCollisions {
    NoCollisions
  }
}

impl CollisionDetector for NoCollisions {
  fn detect(&mut self, _: &CollisionWorldView) -> Vec<CollisionDetail> {
    Vec::new()
  }
}

pub struct SimpleCollisions {
  broad_phase: BruteForceDetector,
  narrow_phase: BruteForceRefiner,
}

impl SimpleCollisions {
  pub fn new() -> SimpleCollisions {
    SimpleCollisions {
      broad_phase: BruteForceDetector::new(),
      narrow_phase: BruteForceRefiner::new()
    }
  }
}

impl CollisionDetector for SimpleCollisions {
  fn detect(&mut self, world: &CollisionWorldView) -> Vec<CollisionDetail> {
    let prelims = self.broad_phase.detect(world);
    self.narrow_phase.refine(world, prelims)
  }
}
