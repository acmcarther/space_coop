use specs;
use engine;

use common::protocol::ClientPayload;
use engine::io::health_check::HealthyEvent;
use engine::player::connection::ConnectEvent;
use engine::player::snapshot::SnapshotAckEvent;
use engine::player::input::InputEvent;

use itertools::Itertools;

/**
 * Directs ClientPayloads to the individual event buses
 *
 * Inputs: ClientPayload
 * Outputs: ConnectEvent, SnapshotAckEvent, ClientEvent, HealthyEvent,
 */
pub struct System;

impl System {
  pub fn new() -> System {
    System
  }
}

impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    use common::protocol::ClientNetworkEvent::*;

    let (mut inbound_events, mut connect_events, mut snapshot_ack_events, mut input_events, mut healthy_events) = arg.fetch(|w| {
      (w.write_resource::<Vec<ClientPayload>>(),
       w.write_resource::<Vec<ConnectEvent>>(),
       w.write_resource::<Vec<SnapshotAckEvent>>(),
       w.write_resource::<Vec<InputEvent>>(),
       w.write_resource::<Vec<HealthyEvent>>())
    });

    // Convert our single message type to several and ship em to different busses
    inbound_events.drain(..).foreach(|payload| {
      healthy_events.push(HealthyEvent::new(payload.address.clone()));
      match payload.event {
        Connect => connect_events.push(ConnectEvent::Connect(payload.address)),
        Disconnect => connect_events.push(ConnectEvent::Disconnect(payload.address)),
        SnapshotAck(idx) => snapshot_ack_events.push(SnapshotAckEvent::new(payload.address, idx)),
        DomainEvent(event) => input_events.push(InputEvent::new(payload.address, event)),
        KeepAlive => () // TODO: pingback svc
      }
    });
  }
}
