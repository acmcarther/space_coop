use Network;
use aspects::PlayerAspect;
use common::protocol::ClientPayload;
use itertools::Itertools;
use protocol::OutboundEvent;
use pubsub::{PubSubStore, SubscriberToken};
use specs;
use state::Delta;

/**
 * Manages the network adapter, broadcasting pending outgoing events and accepting incoming events
 *
 * Input: OutboundEvent, Players
 * Output: InboundEvent
 */
pub struct System {
  network: Network,
  outbound_event_sub_token: SubscriberToken<OutboundEvent>,
}
declare_dependencies!(System, []);

impl System {
  pub fn new(port: u16, world: &mut specs::World) -> System {
    System {
      network: Network::new(port),
      outbound_event_sub_token: world.register_subscriber::<OutboundEvent>(),
    }
  }
}

impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use specs::Join;

    let (mut outbound_events, mut inbound_events, player) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.outbound_event_sub_token).collected(),
       w.fetch_publisher::<ClientPayload>(),
       w.read::<PlayerAspect>())
    });

    let all_addresses = player.iter().map(|player| player.address.clone()).collect();

    // Emit all pending events
    outbound_events.drain(..)
      .flat_map(|outbound| outbound.to_server_payloads(&all_addresses))
      .foreach(|event| self.network.send(event));

    // Process all incoming events
    self.network.recv_pending().into_iter().foreach(|e| inbound_events.push(e));
  }
}
