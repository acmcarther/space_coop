extern crate itertools;
extern crate cgmath;
extern crate specs;
extern crate state;
extern crate pubsub;
extern crate pause;
extern crate glutin;

use cgmath::{Deg, Euler, Quaternion, Rotation, Vector3};
use state::Delta;
use pause::PauseState;
use pubsub::{PubSubStore, SubscriberToken};

#[derive(Debug, Clone)]
pub struct CameraPos(pub f32, pub f32, pub f32);

#[derive(Debug, Clone)]
struct CameraMoveEvent(pub i32);

/**
 * Send the events from the windowing system to event busses
 */
pub struct MovementSystem {
  camera_event_sub_token: SubscriberToken<CameraMoveEvent>,
}

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
  window_event_sub_token: SubscriberToken<glutin::Event>,
}

impl PreprocessorSystem {
  pub fn new(world: &mut specs::World) -> PreprocessorSystem {
    PreprocessorSystem { window_event_sub_token: world.register_subscriber::<glutin::Event>() }
  }

  pub fn name() -> &'static str {
    "camera::preprocessor"
  }
}

impl specs::System<Delta> for PreprocessorSystem {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use itertools::Itertools;
    use glutin::Event::MouseMoved;

    let (mut glutin_events, pause_state, window, mut camera_events) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.window_event_sub_token).collected(),
       w.read_resource::<PauseState>(),
       w.write_resource::<glutin::Window>(),
       w.fetch_publisher::<CameraMoveEvent>())
    });

    if *pause_state != PauseState::Paused {
      glutin_events.drain(..).foreach(|e| {
        match e {
          MouseMoved(x, _) => {
            // Move the mouse back to the middle of the window
            let (wx, wy) = window.get_position().unwrap();
            let (ox, oy) = window.get_outer_size().unwrap();
            let (middle_x, middle_y) = ((wx + ox as i32 / 2), (wy + oy as i32 / 2));
            window.set_cursor_position(middle_x, middle_y).unwrap();

            camera_events.push(CameraMoveEvent(x - middle_x));
          },
          _ => {}, // It's goin' on the ground
        }
      });
    }
  }
}
