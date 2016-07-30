use specs;
use glutin;
use engine;
use itertools::Itertools;
use common::world::{PhysicalAspect, SynchronizedAspect};

use world::{CameraPos, OwnEntity};
use cgmath::{Deg, Euler, Quaternion, Rotation, Vector3};

pub struct CameraMoveEvent(pub i32, pub i32);

/**
 * Send the events from the windowing system to event busses
 */
pub struct System;

impl System {
  pub fn new() -> System {
    System
  }
}

#[allow(unused_imports, unused_variables)]
impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    use specs::Join;
    use itertools::Itertools;

    let (mut camera_events,
         mut camera_pos,
         own_entity,
         physical,
         synchronized,
         entities,
         window) = arg.fetch(|w| {
      (w.write_resource::<Vec<CameraMoveEvent>>(),
       w.write_resource::<CameraPos>(),
       w.read_resource::<Option<OwnEntity>>(),
       w.read::<PhysicalAspect>(),
       w.read::<SynchronizedAspect>(),
       w.entities(),
       w.read_resource::<glutin::Window>())
    });

    // Move the mouse back to the middle of the window
    let (wx, wy) = window.get_position().unwrap();
    let (ox, oy) = window.get_outer_size().unwrap();
    let (middle_x, middle_y) = ((wx + ox as i32 / 2), (wy + oy as i32 / 2));
    window.set_cursor_position(middle_x, middle_y).unwrap();

    let mut camera_manipulator = CameraManipulator::new(&mut camera_pos, middle_x);
    camera_events.drain(..).foreach(|e| camera_manipulator.rotate_camera(e));
  }
}

// TODO(acmcarther): Document
struct CameraManipulator<'a> {
  camera_pos: &'a mut CameraPos,
  window_center_x: i32,
}

impl<'a> CameraManipulator<'a> {
  pub fn new(camera_pos: &'a mut CameraPos, window_center_x: i32) -> CameraManipulator<'a> {
    CameraManipulator {
      camera_pos: camera_pos,
      window_center_x: window_center_x,
    }
  }

  pub fn rotate_camera(&mut self, event: CameraMoveEvent) {
    // Get current camera radius
    let CameraPos(c_x, c_y, c_z) = self.camera_pos.clone();
    let CameraMoveEvent(x, _) = event;

    let rel_x = x - self.window_center_x;
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
