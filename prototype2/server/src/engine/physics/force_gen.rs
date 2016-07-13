use cgmath::Vector3;//, Quaternion};

use common::world::PhysicalAspect;

pub struct Force {
  pub d_position: Option<Vector3<f32>>,
  pub d_velocity: Option<Vector3<f32>>,
  //d_torque: Option<Quaternion<f32>>,
  //d_rotation: Option<Quaternion<f32>>
}

impl Force {
  pub fn with_velocity(vel: Vector3<f32>) -> Force {
    Force {
      d_position: None,
      d_velocity: Some(vel),
      //d_torque: None,
      //d_rotation: None
    }
  }
}

pub trait ForceGenerator {
  fn generate(&mut self, ent: &PhysicalAspect, dt_s: f32) -> Option<Force>;
}

pub struct Gravity;

impl Gravity {
  pub fn new() -> Gravity {
    Gravity
  }
}

impl ForceGenerator for Gravity {
  fn generate(&mut self, _: &PhysicalAspect, dt_s: f32) -> Option<Force> {
    Some(Force::with_velocity(Vector3::<f32>::new(0.0, 0.0, -0.29 * dt_s)))
  }
}

pub struct GroundImpactor;

impl GroundImpactor {
  pub fn new() -> GroundImpactor {
    GroundImpactor
  }
}

impl ForceGenerator for GroundImpactor {
  fn generate(&mut self, ent: &PhysicalAspect, _: f32) -> Option<Force> {
    // Bounce on "ground"
    if ent.pos.2 < 0.0 {
      let force = Force {
        d_velocity: Some(Vector3::<f32>::new(0.0, 0.0, - 1.7 * ent.vel.2)),
        d_position: Some(Vector3::<f32>::new(0.0, 0.0, -ent.pos.2)),
        //d_torque: None,
        //d_rotation: None
      };
      Some(force)
    } else {
      None
    }
  }
}
