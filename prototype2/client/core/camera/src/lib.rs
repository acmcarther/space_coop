#![feature(type_macros)]
extern crate itertools;
extern crate cgmath;
extern crate specs;
extern crate client_state as state;
extern crate pubsub;
extern crate mouse_lock;
#[macro_use(declare_dependencies, standalone_installer_from_new)]
extern crate automatic_system_installer;

use cgmath::{Deg, Euler, Quaternion, Rotation, Vector3};
use mouse_lock::RelativeMouseMovementEvent;
use pubsub::{PubSubStore, SubscriberToken};
use state::Delta;

#[derive(PartialEq, Debug, Clone)]
pub struct CameraPos(pub f32, pub f32, pub f32);

#[derive(Debug, Clone)]
struct CameraMoveEvent(pub i32);

/**
 * Send the events from the windowing system to event busses
 */
pub struct MovementSystem {
  camera_event_sub_token: SubscriberToken<CameraMoveEvent>,
}
declare_dependencies!(MovementSystem, [PreprocessorSystem]);
standalone_installer_from_new!(MovementSystem, Delta);

impl MovementSystem {
  pub fn new(world: &mut specs::World) -> MovementSystem {
    world.add_resource::<CameraPos>(CameraPos(3.0, -10.0, 6.0));
    MovementSystem { camera_event_sub_token: world.register_subscriber::<CameraMoveEvent>() }
  }

  pub fn name() -> &'static str {
    "camera::MovementSystem"
  }
}

impl specs::System<Delta> for MovementSystem {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use itertools::Itertools;

    let (mut camera_events, mut camera_pos) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.camera_event_sub_token).collected(),
       w.write_resource::<CameraPos>())
    });

    let mut camera_manipulator = CameraManipulator::new(&mut camera_pos);
    camera_events.drain(..).foreach(|e| camera_manipulator.rotate_camera(e));
  }
}


// TODO(acmcarther): Document
struct CameraManipulator<'a> {
  camera_pos: &'a mut CameraPos,
}

impl<'a> CameraManipulator<'a> {
  pub fn new(camera_pos: &'a mut CameraPos) -> CameraManipulator<'a> {
    CameraManipulator { camera_pos: camera_pos }
  }

  pub fn rotate_camera(&mut self, event: CameraMoveEvent) {
    // Get current camera radius
    let CameraPos(c_x, c_y, c_z) = self.camera_pos.clone();
    let CameraMoveEvent(rel_x) = event;

    let cam_angle_x = 0.02 * rel_x as f32;

    let rotation = Quaternion::from(Euler {
      x: Deg::new(0.0),
      y: Deg::new(0.0),
      z: Deg::new(cam_angle_x),
    });
    let result = rotation.rotate_vector(Vector3::new(c_x, c_y, c_z));
    *self.camera_pos = CameraPos(result.x, result.y, result.z);
  }
}

pub struct PreprocessorSystem {
  relative_mouse_movement_sub_token: SubscriberToken<RelativeMouseMovementEvent>,
}
declare_dependencies!(PreprocessorSystem, [mouse_lock::System]);
standalone_installer_from_new!(PreprocessorSystem, Delta);

impl PreprocessorSystem {
  pub fn new(world: &mut specs::World) -> PreprocessorSystem {
    PreprocessorSystem {
      relative_mouse_movement_sub_token: world.register_subscriber::<RelativeMouseMovementEvent>(),
    }
  }

  pub fn name() -> &'static str {
    "camera::preprocessor"
  }
}

impl specs::System<Delta> for PreprocessorSystem {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use itertools::Itertools;

    let (mut relative_mouse_movement_events, mut camera_events) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.relative_mouse_movement_sub_token).collected(),
       w.fetch_publisher::<CameraMoveEvent>())
    });

    relative_mouse_movement_events.drain(..).foreach(|e| {
      camera_events.push(CameraMoveEvent(e.x));
    });
  }
}
