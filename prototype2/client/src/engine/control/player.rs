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

    let CameraPos(x, y, _) = *camera_pos;

    // Note: Negative for some reason
    let forward_vec = -Vector2::new(x, y).normalize_to(0.1);
    let perpendicular_vec = -Vector2::new(-y, x).normalize_to(0.1);

    move_events.drain(..).foreach(|e| {
      match e {
        MoveEvent::Forward => {
          outbound_events.push(DomainEvent(SelfMove {
            x_d: forward_vec.x,
            y_d: forward_vec.y,
            z_d: 0.0,
          }))
        },
        MoveEvent::Backward => {
          outbound_events.push(DomainEvent(SelfMove {
            x_d: -forward_vec.x,
            y_d: -forward_vec.y,
            z_d: 0.0,
          }))
        },
        MoveEvent::Left => {
          outbound_events.push(DomainEvent(SelfMove {
            x_d: perpendicular_vec.x,
            y_d: perpendicular_vec.y,
            z_d: 0.0,
          }))
        },
        MoveEvent::Right => {
          outbound_events.push(DomainEvent(SelfMove {
            x_d: -perpendicular_vec.x,
            y_d: -perpendicular_vec.y,
            z_d: 0.0,
          }))
        },
      }
    });
  }
}
