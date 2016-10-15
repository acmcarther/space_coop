extern crate specs;
extern crate ncollide;
extern crate nalgebra;
extern crate nphysics3d;
extern crate itertools;

extern crate common;
extern crate aspects;
extern crate server_state as state;

use aspects::CollisionAspect;
use common::ecs::aspects::{DisabledAspect, PhysicalAspect};
use common::geometry::model::ModelType;
use nalgebra::Rotation;
use nalgebra::Translation;
use ncollide::shape::{Ball, Plane, Cuboid};
use nphysics3d::math::Vector;
use nphysics3d::object::{RigidBody, RigidBodyHandle};
use nphysics3d::world::World;
use state::Delta;
use std::ops::{Deref, DerefMut};

#[no_mangle]
pub fn run_physics_system(arg: specs::RunArg, delta: Delta, world: &mut World<f32>) {
    use specs::Join;
    use itertools::Itertools;
    use std::ops::Not;

    let (mut physicals, collisions, disabled) = arg.fetch(|w| {
        (w.write::<PhysicalAspect>(), w.read::<CollisionAspect>(), w.read::<DisabledAspect>())
    });

    world.set_gravity(Vector::new(0.0, -0.981, 0.0));

    let dt_s = (delta.dt.num_milliseconds() as f32) / 1000.0;
    let sim_objects = (&mut physicals, &collisions, disabled.not())
        .iter()
        .map(|(physical, collision, _)| {

            let mut entity = match collision.model {
                ModelType::Cube => {
                    RigidBody::new_dynamic(Cuboid::new(Vector::new(1.0, 1.0, 1.0)), 1.0, 0.3, 0.6)
                }
                ModelType::Icosphere0 |
                ModelType::Icosphere1 |
                ModelType::Icosphere2 |
                ModelType::Icosphere3 => RigidBody::new_dynamic(Ball::new(1.0), 1.0, 0.3, 0.6),
            };

            entity.append_rotation(&Vector::new(physical.ang.0, physical.ang.1, physical.ang.2));
            entity.append_translation(&Vector::new(physical.pos.0, physical.pos.2, physical.pos.1));
            entity.set_lin_vel(Vector::new(physical.vel.0, physical.vel.2, physical.vel.1));
            entity.set_ang_vel(Vector::new(physical.ang_vel.0,
                                           physical.ang_vel.1,
                                           physical.ang_vel.2));

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
        //
        world.remove_rigid_body(&handle);
    });
}
