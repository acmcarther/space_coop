use specs;
use engine;

use common::world::{
  PhysicalAspect,
  DisabledAspect
};

/**
 * Simulates the world.
 *
 * Inputs: Physicals, Collisions
 * Outputs: Physicals
 */
pub struct System;

impl System {
  pub fn new() -> System {
    System
  }
}

impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, delta: engine::Delta) {
    use specs::Join;
    use itertools::Itertools;
    use std::ops::Not;

    let (mut physicals, disabled) = arg.fetch(|w| {
      (w.write::<PhysicalAspect>(),
       w.read::<DisabledAspect>())
    });

    (&mut physicals, disabled.not()).iter()
      .map(|(physical, _)| physical)
      .foreach(|physical| {
        let dt_s = (delta.dt.num_milliseconds() as f32) / 1000.0;

        // Gravity
        physical.vel.2 -= 0.98 * dt_s;

        // Integrate vel
        let (d_x, d_y, d_z) = (physical.vel.0 * dt_s, physical.vel.1 * dt_s, physical.vel.2 * dt_s);
        physical.pos = (physical.pos.0 + d_x, physical.pos.1 + d_y, physical.pos.2 + d_z);

        // Dont clip into that planet!
        if physical.pos.2 < 0.0 {
          if physical.vel.2 < 0.0 {
            physical.vel.2 = -(0.6 * physical.vel.2);
          }
          physical.pos.2 = 0.0;
        }
      })
  }
}
