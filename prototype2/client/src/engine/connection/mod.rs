use specs;
use engine;
use itertools::Itertools;
use std::ops::Deref;

use time::{self, Tm};

pub enum ConnectionEvent {
  Connected,
  Disconnected,
  KeepAlive,
}

pub enum ConnectionStatus {
  Connected {
    last_message: Tm,
  },
  Disconnected,
}

impl ConnectionStatus {
  pub fn new() -> ConnectionStatus {
    ConnectionStatus::Disconnected
  }
}

/**
 * Handles internal representation of connnection status
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

    let (mut connection_status, mut connection_events) = arg.fetch(|w| {
      (w.write_resource::<ConnectionStatus>(), w.write_resource::<Vec<ConnectionEvent>>())
    });

    connection_events.drain(..).foreach(|event| {
      let mut out_status = None;
      match (connection_status.deref(), event) {
        (&ConnectionStatus::Connected { .. }, ConnectionEvent::Connected) |
        (&ConnectionStatus::Connected { .. }, ConnectionEvent::KeepAlive) |
        (&ConnectionStatus::Disconnected, ConnectionEvent::Connected) => {
          out_status = Some(ConnectionStatus::Connected { last_message: time::now() })
        },
        (&ConnectionStatus::Connected { .. }, ConnectionEvent::Disconnected) => {
          out_status = Some(ConnectionStatus::Disconnected)
        },
        (&ConnectionStatus::Disconnected, ConnectionEvent::KeepAlive) |
        (&ConnectionStatus::Disconnected, ConnectionEvent::Disconnected) => {},
      }

      out_status.map(|new_status| *connection_status = new_status);
    });
  }
}
