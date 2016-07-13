use cgmath::Vector3;

use engine::physics::collision::{DetectedCollision, CollisionDetail, CollisionWorldView};

/**
 * A function to filter sure collisions from a list of possible collisions
 *
 * Expectation is that this detector be accurate, and probably slow
 */
pub trait NarrowPhaseRefiner {
  fn refine(&mut self, world: &CollisionWorldView, possible_collisions: Vec<DetectedCollision>) -> Vec<CollisionDetail>;
}

/**
 * A detector that just matches entities with origin less than 2 (unit?) apart
 *
 * Not a reasonable refiner in the slightest: does not take model geometry into account
 */
pub struct BruteForceRefiner;

impl BruteForceRefiner {
  pub fn new() -> BruteForceRefiner {
    BruteForceRefiner
  }
}

impl NarrowPhaseRefiner for BruteForceRefiner{
  fn refine(&mut self, world: &CollisionWorldView, possible_collisions: Vec<DetectedCollision>) -> Vec<CollisionDetail> {
    possible_collisions.into_iter()
      // Rejects collisions between non-physical entities
      .filter_map(|collision| {
        // Grab (optionally) the first physical aspect
        world.physical.get(&collision.entities.0)
          // Grab (optionally) the second physical aspect
          .and_then(|first_phys| world.physical.get(&collision.entities.1).map(|second_phys| (first_phys, second_phys)))
          // Do collision formulation
          .and_then(|(first_phys, second_phys)| {
            let dist_sq =
              (first_phys.pos.0 - second_phys.pos.0).powi(2)
              + (first_phys.pos.1 - second_phys.pos.1).powi(2)
              + (first_phys.pos.2 - second_phys.pos.2).powi(2);

            if dist_sq <= 4.0 {
              Some(CollisionDetail {
                entities: collision.entities,
                interpenetration_resolution: (Vector3::<f32>::new(0.0, 0.0, 0.0), Vector3::<f32>::new(0.0, 0.0, 0.0)), // None because lazy
                impulses: (Vector3::<f32>::new(0.0, 0.0, 0.0), Vector3::<f32>::new(0.0, 0.0, 0.0)), // None because lazy
                torques: ((Vector3::<f32>::new(1.0, 0.0, 0.0), 0.0), (Vector3::<f32>::new(1.0, 0.0, 0.0), 0.0)) // None because lazy
              })
            } else {
              None
            }
          })
      }).collect()
  }
}

#[cfg(test)]
mod test {
  use std::collections::HashMap;
  use super::*;

  use cgmath::Vector3;
  use uuid::Uuid;
  use common::world::PhysicalAspect;
  use engine::physics::collision::{DetectedCollision, CollisionDetail, CollisionWorldView};

  #[test]
  fn trivially_returns_none() {
    let physical = HashMap::new();
    let collidable = HashMap::new();
    let world_view = CollisionWorldView { physical: &physical, collidable: &collidable };
    let possible_collisions = Vec::new();

    assert_eq!(BruteForceRefiner::new().refine(&world_view, possible_collisions), Vec::new());
  }

  #[test]
  fn drops_non_physical_entities() {
    let physical = HashMap::new();
    let collidable = HashMap::new();
    let world_view = CollisionWorldView { physical: &physical, collidable: &collidable };
    let possible_collisions = vec![
      DetectedCollision { entities: (Uuid::new_v4(), Uuid::new_v4()) }
    ];

    assert_eq!(BruteForceRefiner::new().refine(&world_view, possible_collisions), Vec::new());
  }

  //#[test]
  #[allow(dead_code)]
  fn identifies_trivial_collisions() {
    let colliding_ent_1 = Uuid::new_v4();
    let colliding_ent_2 = Uuid::new_v4();
    let mut physical = HashMap::new();
    physical.insert(colliding_ent_1.clone(), PhysicalAspect {
      pos: (1.0, 0.0, 0.0),
      vel: (0.0, 0.0, 0.0),
      anchored: false
    });
    physical.insert(colliding_ent_2.clone(), PhysicalAspect {
      pos: (0.0, 0.0, 0.0),
      vel: (0.0, 0.0, 0.0),
      anchored: false
    });
    let collidable = HashMap::new();
    let world_view = CollisionWorldView { physical: &physical, collidable: &collidable };
    let possible_collisions = vec![
      DetectedCollision { entities: (colliding_ent_1.clone(), colliding_ent_2.clone()) } ];

    assert_eq!(BruteForceRefiner::new().refine(&world_view, possible_collisions), vec![
      CollisionDetail {
        entities: (colliding_ent_1, colliding_ent_2),
        interpenetration_resolution: (Vector3::<f32>::new(-0.01, 0.0, 0.0), Vector3::<f32>::new(2.01, 0.0, 0.0)),
        impulses: (Vector3::<f32>::new(0.0, 0.0, 0.0), Vector3::<f32>::new(0.0, 0.0, 0.0)),
        torques: ((Vector3::<f32>::new(1.0, 0.0, 0.0), 0.0), (Vector3::<f32>::new(1.0, 0.0, 0.0), 0.0)),
      }
    ]);
  }

  //#[test]
  #[allow(dead_code)]
  fn identifies_near_collision() {
    let colliding_ent_1 = Uuid::new_v4();
    let colliding_ent_2 = Uuid::new_v4();
    let mut physical = HashMap::new();
    physical.insert(colliding_ent_1.clone(), PhysicalAspect {
      pos: (2.0, 0.0, 0.0),
      vel: (-1.0, 0.0, 0.0),
      anchored: false
    });
    physical.insert(colliding_ent_2.clone(), PhysicalAspect {
      pos: (0.0, 0.0, 0.0),
      vel: (1.0, 0.0, 0.0),
      anchored: false
    });
    let collidable = HashMap::new();
    let world_view = CollisionWorldView { physical: &physical, collidable: &collidable };
    let possible_collisions = vec![
      DetectedCollision { entities: (colliding_ent_1.clone(), colliding_ent_2.clone()) } ];

    assert_eq!(BruteForceRefiner::new().refine(&world_view, possible_collisions), vec![
      CollisionDetail {
        entities: (colliding_ent_1, colliding_ent_2),
        interpenetration_resolution: (Vector3::<f32>::new(-0.01, 0.0, 0.0), Vector3::<f32>::new(2.01, 0.0, 0.0)),
        impulses: (Vector3::<f32>::new(1.0, 0.0, 0.0), Vector3::<f32>::new(-1.0, 0.0, 0.0)),
        torques: ((Vector3::<f32>::new(1.0, 0.0, 0.0), 0.0), (Vector3::<f32>::new(1.0, 0.0, 0.0), 0.0)),
      }
    ]);
  }

  #[test]
  fn drops_near_miss() {
    let colliding_ent_1 = Uuid::new_v4();
    let colliding_ent_2 = Uuid::new_v4();
    let mut physical = HashMap::new();
    physical.insert(colliding_ent_1.clone(), PhysicalAspect {
      pos: (2.1, 0.0, 0.0),
      vel: (0.0, 0.0, 0.0),
      anchored: false
    });
    physical.insert(colliding_ent_2.clone(), PhysicalAspect {
      pos: (0.0, 0.0, 0.0),
      vel: (0.0, 0.0, 0.0),
      anchored: false
    });
    let collidable = HashMap::new();
    let world_view = CollisionWorldView { physical: &physical, collidable: &collidable };
    let possible_collisions = vec![
      DetectedCollision { entities: (colliding_ent_1.clone(), colliding_ent_2.clone()) } ];

    assert_eq!(BruteForceRefiner::new().refine(&world_view, possible_collisions), Vec::new());
  }
}

