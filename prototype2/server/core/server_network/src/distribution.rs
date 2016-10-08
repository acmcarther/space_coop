use common::protocol::{ClientPayload, ClientEvent};
use itertools::Itertools;
use pubsub::{PubSubStore, SubscriberToken};
use specs;
use state::Delta;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub enum ConnectEvent {
  Connect(SocketAddr),
  Disconnect(SocketAddr),
}

#[derive(Debug, Clone)]
pub struct HealthyEvent(SocketAddr);

impl HealthyEvent {
  pub fn new(address: SocketAddr) -> HealthyEvent {
    HealthyEvent(address)
  }

  pub fn address(&self) -> &SocketAddr {
    let &HealthyEvent(ref addr) = self;
    addr
  }
}

#[derive(Debug, Clone)]
pub struct InputEvent {
  pub address: SocketAddr,
  pub event: ClientEvent,
}

impl InputEvent {
  pub fn new(address: SocketAddr, event: ClientEvent) -> InputEvent {
    InputEvent {
      address: address,
      event: event,
    }
  }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SnapshotAckEvent {
  address: SocketAddr,
  idx: u16,
}

impl SnapshotAckEvent {
  pub fn new(address: SocketAddr, idx: u16) -> SnapshotAckEvent {
    SnapshotAckEvent {
      address: address,
      idx: idx,
    }
  }
}


/**
 * Directs ClientPayloads to the individual event buses
 *
 * Inputs: ClientPayload
 * Outputs: ConnectEvent, SnapshotAckEvent, ClientEvent, HealthyEvent,
 */
pub struct System {
  client_payload_sub_token: SubscriberToken<ClientPayload>,
}
declare_dependencies!(System, [::adapter::System]);
standalone_installer_from_new!(System, Delta);

impl System {
  pub fn new(world: &mut specs::World) -> System {
    System { client_payload_sub_token: world.register_subscriber() }
  }
}

impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use common::protocol::ClientNetworkEvent::*;

    let (mut inbound_events,
         mut connect_events,
         mut snapshot_ack_events,
         mut input_events,
         mut healthy_events) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.client_payload_sub_token).collected(),
       w.fetch_publisher::<ConnectEvent>(),
       w.fetch_publisher::<SnapshotAckEvent>(),
       w.fetch_publisher::<InputEvent>(),
       w.fetch_publisher::<HealthyEvent>())
    });

    // Convert our single message type to several and ship em to different busses
    inbound_events.drain(..).foreach(|payload| {
      healthy_events.push(HealthyEvent::new(payload.address.clone()));
      match payload.event {
        Connect => connect_events.push(ConnectEvent::Connect(payload.address)),
        Disconnect => connect_events.push(ConnectEvent::Disconnect(payload.address)),
        SnapshotAck(idx) => snapshot_ack_events.push(SnapshotAckEvent::new(payload.address, idx)),
        DomainEvent(event) => input_events.push(InputEvent::new(payload.address, event)),
        KeepAlive => (), // TODO: pingback svc
      }
    });
  }
}
