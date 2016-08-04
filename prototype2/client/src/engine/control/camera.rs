use specs;
use glutin;
use engine;
use itertools::Itertools;

use world::CameraPos;
use cgmath::{Deg, Euler, Quaternion, Rotation, Vector3};

pub struct CameraMoveEvent(pub i32);

/**
 * Send the events from the windowing system to event busses
 */
pub struct System;

impl System {
  pub fn new() -> System {
    System
  }
}

impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    use itertools::Itertools;

    let (mut camera_events, mut camera_pos, window) = arg.fetch(|w| {
      (w.write_resource::<Vec<CameraMoveEvent>>(),
       w.write_resource::<CameraPos>(),
       w.read_resource::<glutin::Window>())
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
