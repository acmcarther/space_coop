use specs;

use engine;

use itertools::Itertools;

use engine::connection::ConnectionEvent;

/**
 * Directs ServerNetworkEvents to the individual event buses
 */
pub struct System;

impl System {
  pub fn new() -> System {
    System
  }
}

impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    use common::protocol::ServerNetworkEvent;
    use common::protocol::SnapshotEvent;
    use common::protocol::ServerNetworkEvent::*;

    let (mut inbound_events, mut connection_events, mut snapshot_events) = arg.fetch(|w| {
      (w.write_resource::<Vec<ServerNetworkEvent>>(),
       w.write_resource::<Vec<ConnectionEvent>>(),
       w.write_resource::<Vec<SnapshotEvent>>())
    });

    inbound_events.drain(..).foreach(|e| {
      match e {
        Connected => connection_events.push(ConnectionEvent::Connected),
        Disconnected => connection_events.push(ConnectionEvent::Disconnected),
        KeepAlive => connection_events.push(ConnectionEvent::KeepAlive),
        Error(msg) => println!("Server Error?: {}", msg),
        Snapshot(event) => snapshot_events.push(event),
      }
    });
  }
}
