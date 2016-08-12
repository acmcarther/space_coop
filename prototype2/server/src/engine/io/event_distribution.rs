use specs;

use common::protocol::ClientPayload;
use player::{ConnectEvent, HealthyEvent, InputEvent, SnapshotAckEvent};
use pubsub::{PubSubStore, SubscriberToken};
use state::Delta;

use itertools::Itertools;

/**
 * Directs ClientPayloads to the individual event buses
 *
 * Inputs: ClientPayload
 * Outputs: ConnectEvent, SnapshotAckEvent, ClientEvent, HealthyEvent,
 */
pub struct System {
  client_payload_sub_token: SubscriberToken<ClientPayload>,
}

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
