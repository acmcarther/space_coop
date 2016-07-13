use uuid::Uuid;

use engine::physics::collision::{DetectedCollision, CollisionWorldView};

/**
 * A function to identify "possible" collisions
 *
 * Expectation is that this detector be fast, and probably yield some false positives
 */
pub trait BroadPhaseDetector {
  fn detect(&mut self, world: &CollisionWorldView) -> Vec<DetectedCollision>;
}

/**
 * A detector that just matches all possible pairs
 *
 * A very fast detector, that likely makes the collision detection slow because it gives too much
 * work to the refiner.
 */
pub struct BruteForceDetector;

impl BruteForceDetector {
  pub fn new() -> BruteForceDetector {
    BruteForceDetector
  }
}

impl BroadPhaseDetector for BruteForceDetector {
  fn detect(&mut self, world: &CollisionWorldView) -> Vec<DetectedCollision> {
    // Pairwise match everything
    world.collidable.keys().cloned()
      .enumerate()
      .flat_map(|(idx, entity)| world.collidable.keys().cloned().skip(idx + 1).map(|other| (entity.clone(), other)).collect::<Vec<(Uuid, Uuid)>>().into_iter())
      .map(|(x, y)| DetectedCollision { entities: (x, y) })
      .collect()
  }
}

#[cfg(test)]
mod test {
  use std::collections::HashMap;
  use super::*;
  use uuid::Uuid;
  use engine::physics::collision::{DetectedCollision, CollisionWorldView};
  use world::CollisionAspect;

  #[test]
  fn returns_trivial_pairwise_collisions() {
    let physical = HashMap::new();
    let mut collidable = HashMap::new();
    let colliding_ent_1 = Uuid::new_v4();
    let colliding_ent_2 = Uuid::new_v4();
    collidable.insert(colliding_ent_1, CollisionAspect::new());
    collidable.insert(colliding_ent_2, CollisionAspect::new());
    let world_view = CollisionWorldView { physical: &physical, collidable: &collidable };

    let result = BruteForceDetector::new().detect(&world_view);
    assert_eq!(result.len(), 1);
    assert!(result.contains(&DetectedCollision { entities: (colliding_ent_1.clone(), colliding_ent_2.clone()) })
        || result.contains(&DetectedCollision { entities: (colliding_ent_2.clone(), colliding_ent_1.clone()) }));
  }

  #[test]
  fn returns_several_pairwise_collisions() {
    let physical = HashMap::new();
    let mut collidable = HashMap::new();
    let colliding_ent_1 = Uuid::new_v4();
    let colliding_ent_2 = Uuid::new_v4();
    let colliding_ent_3 = Uuid::new_v4();
    collidable.insert(colliding_ent_1, CollisionAspect::new());
    collidable.insert(colliding_ent_2, CollisionAspect::new());
    collidable.insert(colliding_ent_3, CollisionAspect::new());
    let world_view = CollisionWorldView { physical: &physical, collidable: &collidable };

    let result = BruteForceDetector::new().detect(&world_view);
    assert_eq!(result.len(), 3);
    assert!(result.contains(&DetectedCollision { entities: (colliding_ent_1.clone(), colliding_ent_2.clone()) })
        || result.contains(&DetectedCollision { entities: (colliding_ent_2.clone(), colliding_ent_1.clone()) }));
    assert!(result.contains(&DetectedCollision { entities: (colliding_ent_1.clone(), colliding_ent_3.clone()) })
        || result.contains(&DetectedCollision { entities: (colliding_ent_3.clone(), colliding_ent_1.clone()) }));
    assert!(result.contains(&DetectedCollision { entities: (colliding_ent_2.clone(), colliding_ent_3.clone()) })
        || result.contains(&DetectedCollision { entities: (colliding_ent_3.clone(), colliding_ent_2.clone()) }));
  }

  #[test]
  fn returns_complex_pairwise_collisions() {
    let physical = HashMap::new();
    let mut collidable = HashMap::new();
    let colliding_ent_1 = Uuid::new_v4();
    let colliding_ent_2 = Uuid::new_v4();
    let colliding_ent_3 = Uuid::new_v4();
    let colliding_ent_4 = Uuid::new_v4();
    collidable.insert(colliding_ent_1, CollisionAspect::new());
    collidable.insert(colliding_ent_2, CollisionAspect::new());
    collidable.insert(colliding_ent_3, CollisionAspect::new());
    collidable.insert(colliding_ent_4, CollisionAspect::new());
    let world_view = CollisionWorldView { physical: &physical, collidable: &collidable };

    assert_eq!(BruteForceDetector::new().detect(&world_view).len(), 6);
  }
}
