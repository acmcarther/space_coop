extern crate specs;
extern crate ncollide;
extern crate nalgebra;
extern crate nphysics3d;
extern crate itertools;

extern crate common;
extern crate aspects;
extern crate server_state as state;
extern crate pubsub;
extern crate libloading;

#[macro_use(declare_dependencies, standalone_installer_from_new)]
extern crate automatic_system_installer;

use libloading::Library;
use ncollide::shape::Plane;
use nphysics3d::math::Vector;
use nphysics3d::object::RigidBody;
use nphysics3d::world::World;
use state::Delta;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::time::SystemTime;

/**
 * Simulates the world.
 *
 * Inputs: Physicals, Collisions
 * Outputs: Physicals
 */
pub struct System {
  world: World<f32>,
  physics_dylib: Option<Library>,
  physics_dylib_last_modified: SystemTime,
}
declare_dependencies!(System, []);
standalone_installer_from_new!(System, Delta);

const LIB_PATH: &'static str = "../physics_dylib/target/debug/libphysics_dylib.so";

// Lets us make physics sync
pub struct InternalWorld(World<f32>);

// System must never be used outside specs
// NOTE: This gets around nphysics::world::World not being send
//
// Even though we're sending this across thread boundaries, theres never more
// than one thread
// posessing the types in question (Rc).
//
// Jank as heck -- I wish physics used Arc
unsafe impl Send for System {}

impl Deref for InternalWorld {
  type Target = World<f32>;
  fn deref(&self) -> &World<f32> {
    &self.0
  }
}

impl DerefMut for InternalWorld {
  fn deref_mut(&mut self) -> &mut World<f32> {
    &mut self.0
  }
}

impl System {
  pub fn new(_: &mut specs::World) -> System {
    // Configure world
    let mut world = World::new();
    world.set_gravity(Vector::new(0.0, -0.981, 0.0));

    // Add base plane
    let plane_geometry = Plane::new(Vector::new(0.0, 1.0, 0.0));
    let plane = RigidBody::new_static(plane_geometry, 0.3, 0.6);

    world.add_rigid_body(plane);

    let mut physics_dylib = Library::new(LIB_PATH).unwrap();

    System {
      world: world,
      physics_dylib: Some(physics_dylib),
      physics_dylib_last_modified: SystemTime::now(),
    }
  }

  fn try_reload(&mut self) {
    let mut last_modified = fs::metadata(LIB_PATH).unwrap().modified().unwrap();

    if last_modified > self.physics_dylib_last_modified {
      let physics_dylib = self.physics_dylib.take();
      drop(physics_dylib);

      self.physics_dylib = Some(Library::new(LIB_PATH).unwrap());
    } else {
    }
  }
}

impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, delta: Delta) {
    self.try_reload();

    match self.physics_dylib {
      None => panic!("No physics dylib"),
      Some(ref mut dylib) => {
        let dylib_fn = unsafe {
          dylib.get::<fn(specs::RunArg, Delta, &mut World<f32>)>(b"run_physics_system").unwrap()
        };
        dylib_fn(arg, delta, &mut self.world);
      },
    }
  }
}
