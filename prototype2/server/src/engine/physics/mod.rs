use specs;
use engine;

use common::world::{DisabledAspect, PhysicalAspect};
use ncollide::shape::{Cuboid, Plane};
use nalgebra::Translation;
use nalgebra::Rotation;
use nphysics3d::world::World;
use nphysics3d::object::{RigidBody, RigidBodyHandle};
use nphysics3d::math::Vector;

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

    let (mut physicals, disabled) =
      arg.fetch(|w| (w.write::<PhysicalAspect>(), w.read::<DisabledAspect>()));

    // Configure world
    let mut world = World::new();
    world.set_gravity(Vector::new(0.0, -0.981, 0.0));

    // Add base plane
    let plane_geometry = Plane::new(Vector::new(0.0, 1.0, 0.0));
    let mut plane = RigidBody::new_static(plane_geometry, 0.3, 0.6);

    world.add_rigid_body(plane);

    let dt_s = (delta.dt.num_milliseconds() as f32) / 1000.0;
    let sim_objects = (&mut physicals, disabled.not())
      .iter()
      .map(|(physical, _)| {

        let mut entity = RigidBody::new_dynamic(Cuboid::new(Vector::new(1.0, 1.0, 1.0)), 1.0, 0.3, 0.6);

        entity.append_rotation(&Vector::new(physical.ang.0, physical.ang.1, physical.ang.2));
        entity.append_translation(&Vector::new(physical.pos.0, physical.pos.2, physical.pos.1));
        entity.set_lin_vel(Vector::new(physical.vel.0, physical.vel.2, physical.vel.1));
        entity.set_ang_vel(Vector::new(physical.ang_vel.0, physical.ang_vel.1, physical.ang_vel.2));

        let handle = world.add_rigid_body(entity);
        (physical, handle)
      })
      .collect::<Vec<(&mut PhysicalAspect, RigidBodyHandle<f32>)>>();

      world.step(dt_s);

      sim_objects.into_iter().foreach(|(aspect, handle)| {
        aspect.vel.0 = handle.borrow().lin_vel().translation().x;
        aspect.vel.1 = handle.borrow().lin_vel().translation().z;
        aspect.vel.2 = handle.borrow().lin_vel().translation().y;

        aspect.pos.0 = handle.borrow().position().translation().x;
        aspect.pos.1 = handle.borrow().position().translation().z;
        aspect.pos.2 = handle.borrow().position().translation().y;

        aspect.ang_vel.0 = handle.borrow().ang_vel().x;
        aspect.ang_vel.1 = handle.borrow().ang_vel().y;
        aspect.ang_vel.2 = handle.borrow().ang_vel().z;

        aspect.ang.0 = handle.borrow().position().rotation().x;
        aspect.ang.1 = handle.borrow().position().rotation().y;
        aspect.ang.2 = handle.borrow().position().rotation().z;
        /**/
      });

  }
}
