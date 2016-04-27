use time;
use itertools::Itertools;

use server::engine::Engine;

pub trait Physics {
  fn tick_physics(&mut self, dt: &time::Duration);
}

impl Physics for Engine {
  // Super simple physics: just gravity to y = 0
  fn tick_physics(&mut self, dt: &time::Duration) {
    let physical = &mut self.world.physical;
    let disabled = &mut self.world.disabled;
    physical.iter_mut().foreach(|(uuid, val)| {
      if !disabled.contains(uuid) {
        let dt_s = (dt.num_milliseconds() as f32) / 1000.0;

        // Add gravity
        val.vel.2 = val.vel.2 + (-0.29 * dt_s);

        // Integrate vel
        let (d_x, d_y, d_z) = (val.vel.0 * dt_s, val.vel.1 * dt_s, val.vel.2 * dt_s);
        val.pos = (val.pos.0 + d_x, val.pos.1 + d_y, val.pos.2 + d_z);

        // Bounce on "ground"
        if val.pos.2 < 0.0 {
          val.vel.2 = - 0.7 * val.vel.2;
          val.pos.2 = 0.0;
        }
      }
    })
  }
}
