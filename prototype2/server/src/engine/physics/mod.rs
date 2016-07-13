mod collision;
mod force_gen;

use time;
use itertools::Itertools;

use engine::physics::collision::{CollisionWorldView, CollisionDetector, NoCollisions, SimpleCollisions};
use engine::physics::force_gen::{Force, ForceGenerator, Gravity, GroundImpactor};
use world::ServerWorld;

pub type SimplePhysics = Physics<SimpleCollisions>;
pub type CollisionlessPhysics = Physics<NoCollisions>;

pub struct Physics<C: CollisionDetector> {
  collision_detector: C,
  force_gens: Vec<Box<ForceGenerator>>
}

impl Physics<NoCollisions> {
  #[allow(dead_code)]
  pub fn collisionless_physics() -> CollisionlessPhysics {
    Physics {
      collision_detector: NoCollisions::new(),
      force_gens: vec![
        Box::new(Gravity::new()),
        Box::new(GroundImpactor::new())
      ]
    }
  }
}


impl Physics<SimpleCollisions> {
  pub fn simple_physics() -> SimplePhysics {
    Physics {
      collision_detector: SimpleCollisions::new(),
      force_gens: vec![
        Box::new(Gravity::new()),
        Box::new(GroundImpactor::new())
      ]
    }
  }
}

impl <C: CollisionDetector> Physics<C> {
  // Super simple physics: just gravity to y = 0
  pub fn tick(&mut self, world: &mut ServerWorld, dt: &time::Duration) {
    let _ = self.collision_detector.detect(&CollisionWorldView::from_server_world(world));

    let physical = &mut world.physical;
    let disabled = &world.disabled;
    physical.iter_mut()
      .filter(|&(uuid, _)| !disabled.contains(uuid))
      .foreach(|(_, val)| {
        let dt_s = (dt.num_milliseconds() as f32) / 1000.0;

        self.force_gens.iter_mut()
          .filter_map(|gen| gen.generate(val, dt_s))
          // Dodge borrow checker
          .collect::<Vec<Force>>()
          .into_iter()
          .foreach(|force| {
            force.d_position.map(|d_pos| {
              val.pos.0 += d_pos.x;
              val.pos.1 += d_pos.y;
              val.pos.2 += d_pos.z;
            });
            force.d_velocity.map(|d_vel| {
              val.vel.0 += d_vel.x;
              val.vel.1 += d_vel.y;
              val.vel.2 += d_vel.z;
            });
            // TODO: Handle torque and rot
          });

        // Integrate vel
        let (d_x, d_y, d_z) = (val.vel.0 * dt_s, val.vel.1 * dt_s, val.vel.2 * dt_s);
        val.pos = (val.pos.0 + d_x, val.pos.1 + d_y, val.pos.2 + d_z);
      })
  }
}
