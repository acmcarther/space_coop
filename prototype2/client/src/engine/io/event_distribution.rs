use specs;

use engine;

use itertools::Itertools;

use engine::connection::ConnectionEvent;
use common::protocol::ServerNetworkEvent;
use common::protocol::SnapshotEvent;

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

    let (mut inbound_events, mut connection_events, mut snapshot_events) = arg.fetch(|w| {
      (w.write_resource::<Vec<ServerNetworkEvent>>(),
       w.write_resource::<Vec<ConnectionEvent>>(),
       w.write_resource::<Vec<SnapshotEvent>>())
    });

    let mut router = EventRouter::new(&mut connection_events, &mut snapshot_events);
    inbound_events.drain(..).foreach(|e| router.route_network_event(e));
  }
}

// TODO: Document
struct EventRouter<'a> {
  connection_events: &'a mut Vec<ConnectionEvent>,
  snapshot_events: &'a mut Vec<SnapshotEvent>,
}

impl<'a> EventRouter<'a> {
  pub fn new(connection_events: &'a mut Vec<ConnectionEvent>,
             snapshot_events: &'a mut Vec<SnapshotEvent>)
             -> EventRouter<'a> {
    EventRouter {
      connection_events: connection_events,
      snapshot_events: snapshot_events,
    }
  }

  pub fn route_network_event(&mut self, network_event: ServerNetworkEvent) {
    use common::protocol::ServerNetworkEvent::*;

    match network_event {
      Connected => self.connection_events.push(ConnectionEvent::Connected),
      Disconnected => self.connection_events.push(ConnectionEvent::Disconnected),
      KeepAlive => self.connection_events.push(ConnectionEvent::KeepAlive),
      Error(msg) => println!("Server Error?: {}", msg),
      Snapshot(event) => self.snapshot_events.push(event),
    }
  }
}
