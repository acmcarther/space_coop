extern crate specs;
extern crate cgmath;
extern crate itertools;
extern crate common;
extern crate camera;
extern crate pubsub;
extern crate glutin;
extern crate pause;
extern crate client_state as state;
#[macro_use(declare_dependencies, standalone_installer_from_new)]
extern crate automatic_system_installer;

use cgmath::{InnerSpace, Vector2};
use common::protocol::ClientEvent::SelfMove;
use common::protocol::ClientNetworkEvent::{self, DomainEvent};
use pause::PauseState;
use pubsub::{PubSubStore, Publisher, SubscriberToken};
use state::Delta;

#[derive(Clone)]
enum MoveEvent {
  Forward,
  Backward,
  Left,
  Right,
}

/**
 * Convert client player actions to network events
 */
pub struct MoveSystem {
  move_event_sub_token: SubscriberToken<MoveEvent>,
}
declare_dependencies!(MoveSystem, [PreprocessorSystem, camera::MovementSystem]);
standalone_installer_from_new!(MoveSystem, Delta);

impl MoveSystem {
  pub fn new(world: &mut specs::World) -> MoveSystem {
    MoveSystem { move_event_sub_token: world.register_subscriber::<MoveEvent>() }
  }
}

#[allow(unused_imports, unused_variables)]
impl specs::System<Delta> for MoveSystem {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use specs::Join;
    use itertools::Itertools;

    let (mut move_events, outbound_events, camera_pos) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.move_event_sub_token).collected(),
       w.fetch_publisher::<ClientNetworkEvent>(),
       w.read_resource::<camera::CameraPos>())
    });

    let mut player_manipulator = PlayerManipulator::new(camera_pos.clone(), outbound_events);
    move_events.drain(..).foreach(|e| player_manipulator.move_player(e));
  }
}

// TODO(acmcarther): Document
struct PlayerManipulator<'a> {
  forward_vec: Vector2<f32>,
  left_vec: Vector2<f32>,
  outbound_events: Publisher<'a, ClientNetworkEvent>,
}

impl<'a> PlayerManipulator<'a> {
  pub fn new(cam_pos: camera::CameraPos,
             outbound_events: Publisher<'a, ClientNetworkEvent>)
             -> PlayerManipulator<'a> {
    let camera::CameraPos(x, y, _) = cam_pos;

    PlayerManipulator {
      // Negative because we're looking toward the origin
      forward_vec: -Vector2::new(x, y).normalize_to(0.1),
      left_vec: -Vector2::new(-y, x).normalize_to(0.1),
      outbound_events: outbound_events,
    }
  }

  pub fn move_player(&mut self, event: MoveEvent) {
    let move_dir = match event {
      MoveEvent::Forward => self.forward_vec.clone(),
      MoveEvent::Backward => -self.forward_vec.clone(),
      MoveEvent::Left => self.left_vec.clone(),
      MoveEvent::Right => -self.left_vec.clone(),
    };

    self.moove(move_dir);
  }

  fn moove(&mut self, dir: Vector2<f32>) {
    let (x, y) = (dir.x, dir.y);
    self.outbound_events.push(DomainEvent(SelfMove {
      x_d: x,
      y_d: y,
      z_d: 0.0,
    }))
  }
}

pub struct PreprocessorSystem {
  window_event_sub_token: SubscriberToken<glutin::Event>,
}
// NOTE: This depends on a window emitter that lives in the main thread
declare_dependencies!(PreprocessorSystem, [pause::System]);
standalone_installer_from_new!(PreprocessorSystem, Delta);

impl PreprocessorSystem {
  pub fn new(world: &mut specs::World) -> PreprocessorSystem {
    PreprocessorSystem { window_event_sub_token: world.register_subscriber::<glutin::Event>() }
  }
}

impl specs::System<Delta> for PreprocessorSystem {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use itertools::Itertools;
    use glutin::VirtualKeyCode::{A, D, S, W};
    use glutin::ElementState;
    use glutin::Event::KeyboardInput;

    let (mut glutin_events, pause_state, mut move_events) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.window_event_sub_token).collected(),
       w.read_resource::<PauseState>(),
       w.fetch_publisher::<MoveEvent>())
    });

    if *pause_state != PauseState::Paused {
      glutin_events.drain(..).foreach(|e| {
        match e {
          KeyboardInput(ElementState::Pressed, _, Some(W)) => {
            move_events.push(MoveEvent::Forward);
          },
          KeyboardInput(ElementState::Pressed, _, Some(A)) => {
            move_events.push(MoveEvent::Left);
          },
          KeyboardInput(ElementState::Pressed, _, Some(S)) => {
            move_events.push(MoveEvent::Backward);
          },
          KeyboardInput(ElementState::Pressed, _, Some(D)) => {
            move_events.push(MoveEvent::Right);
          },
          _ => {}, // I throw this on the ground
        }
      });
    }
  }
}
