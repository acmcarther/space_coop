use specs;
use engine;

use common::protocol::ClientNetworkEvent::{self, DomainEvent};
use common::protocol::ClientEvent::SelfMove;
use world::CameraPos;
use cgmath::{InnerSpace, Vector2};

pub enum MoveEvent {
  Forward,
  Backward,
  Left,
  Right,
}

/**
 * Convert client player actions to network events
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

    let (mut move_events, mut outbound_events, camera_pos) = arg.fetch(|w| {
      (w.write_resource::<Vec<MoveEvent>>(),
       w.write_resource::<Vec<ClientNetworkEvent>>(),
       w.read_resource::<CameraPos>())
    });

    let mut player_manipulator = PlayerManipulator::new(camera_pos.clone(), &mut outbound_events);
    move_events.drain(..).foreach(|e| player_manipulator.move_player(e));
  }
}

// TODO(acmcarther): Document
struct PlayerManipulator<'a> {
  forward_vec: Vector2<f32>,
  left_vec: Vector2<f32>,
  outbound_events: &'a mut Vec<ClientNetworkEvent>,
}

impl<'a> PlayerManipulator<'a> {
  pub fn new(cam_pos: CameraPos,
             outbound_events: &'a mut Vec<ClientNetworkEvent>)
             -> PlayerManipulator<'a> {
    let CameraPos(x, y, _) = cam_pos;

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
