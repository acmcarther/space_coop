use specs;

use common::Delta;
use time::{self, Duration, Tm};
use Network;
use std::net::SocketAddr;
use std::ops::Deref;
use common::protocol::{ClientNetworkEvent, ServerNetworkEvent, SnapshotEvent};
use std::sync::mpsc::Receiver;

use itertools::Itertools;

use pubsub::{PubSubStore, Publisher, SubscriberToken};

#[derive(Clone)]
pub enum ConnectionEvent {
  Connected,
  Disconnected,
  KeepAlive,
}

#[derive(Clone)]
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
 * Manages the network adapter, broadcasting pending outgoing events and accepting incoming events
 *
 * Also handles telling the server we're disconnecting.
 *
 * Input: ServerNetworkEvent,
 * Output: ClientNetworkEvent
 */
pub struct AdapterSystem {
  network: Network,
  network_kill_signal: Receiver<()>,
  client_event_sub_token: SubscriberToken<ClientNetworkEvent>,
}

impl AdapterSystem {
  pub fn new(port: u16,
             server_addr: SocketAddr,
             network_kill_signal: Receiver<()>,
             world: &mut specs::World)
             -> AdapterSystem {
    world.add_resource::<ConnectionStatus>(ConnectionStatus::new());
    let mut network = Network::new(port, server_addr);
    let success = network.connect();
    if !success {
      println!("Could not connect to {}", server_addr.to_string());
      panic!("could not connect, i need to make this not out of band");
    }
    AdapterSystem {
      network: network,
      network_kill_signal: network_kill_signal,
      client_event_sub_token: world.register_subscriber::<ClientNetworkEvent>(),
    }
  }

  pub fn name() -> &'static str {
    "network::AdapterSystem"
  }
}

impl specs::System<Delta> for AdapterSystem {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    let (mut outbound_events, mut inbound_events) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.client_event_sub_token).collected(),
       w.fetch_publisher::<ServerNetworkEvent>())
    });

    outbound_events.drain(..)
      .foreach(|event| self.network.send(event));

    self.network.recv_pending().into_iter().foreach(|e| inbound_events.push(e));

    if let Ok(()) = self.network_kill_signal.try_recv() {
      self.network.disconnect();
    }
  }
}

/**
 * Handles internal representation of connnection status
 */
pub struct ConnectionSystem {
  connection_event_sub_token: SubscriberToken<ConnectionEvent>,
}

impl ConnectionSystem {
  pub fn new(world: &mut specs::World) -> ConnectionSystem {
    ConnectionSystem { connection_event_sub_token: world.register_subscriber::<ConnectionEvent>() }
  }

  pub fn name() -> &'static str {
    "network::ConnectionSystem"
  }
}

#[allow(unused_imports, unused_variables)]
impl specs::System<Delta> for ConnectionSystem {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use specs::Join;
    use itertools::Itertools;

    let (mut connection_status, mut connection_events) = arg.fetch(|w| {
      (w.write_resource::<ConnectionStatus>(),
       w.fetch_subscriber(&self.connection_event_sub_token).collected())
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

/**
 * Ping back to notify server we're still alive
 */
pub struct KeepAliveSystem {
  next_keepalive_time: Tm,
}

impl KeepAliveSystem {
  pub fn new(_: &mut specs::World) -> KeepAliveSystem {
    KeepAliveSystem { next_keepalive_time: time::now() }
  }

  pub fn name() -> &'static str {
    "network::KeepAliveSystem"
  }
}

#[allow(unused_imports, unused_variables)]
impl specs::System<Delta> for KeepAliveSystem {
  fn run(&mut self, arg: specs::RunArg, delta: Delta) {
    use specs::Join;
    use itertools::Itertools;

    let mut outbound_events = arg.fetch(|w| w.fetch_publisher::<ClientNetworkEvent>());

    if delta.now > self.next_keepalive_time {
      outbound_events.push(ClientNetworkEvent::KeepAlive);
      self.next_keepalive_time = delta.now + Duration::milliseconds(20);
    }
  }
}

/**
 * Directs the internal network events to handler systems
 */
pub struct EventDistributionSystem {
  server_event_sub_token: SubscriberToken<ServerNetworkEvent>,
}

impl EventDistributionSystem {
  pub fn new(world: &mut specs::World) -> EventDistributionSystem {
    EventDistributionSystem {
      server_event_sub_token: world.register_subscriber::<ServerNetworkEvent>(),
    }
  }

  pub fn name() -> &'static str {
    "network::EventDistributionSystem"
  }
}

impl specs::System<Delta> for EventDistributionSystem {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {

    let (mut inbound_events, connection_events, snapshot_events) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.server_event_sub_token).collected(),
       w.fetch_publisher::<ConnectionEvent>(),
       w.fetch_publisher::<SnapshotEvent>())
    });

    let mut router = EventRouter::new(connection_events, snapshot_events);
    inbound_events.drain(..).foreach(|e| router.route_network_event(e));
  }
}

// TODO: Document
struct EventRouter<'a> {
  connection_events: Publisher<'a, ConnectionEvent>,
  snapshot_events: Publisher<'a, SnapshotEvent>,
}

impl<'a> EventRouter<'a> {
  pub fn new(connection_events: Publisher<'a, ConnectionEvent>,
             snapshot_events: Publisher<'a, SnapshotEvent>)
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
