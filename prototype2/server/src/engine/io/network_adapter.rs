use specs;
use engine;

use network::Network;

use common::protocol::ClientPayload;
use world::PlayerAspect;
use protocol::OutboundEvent;

use itertools::Itertools;

/**
 * Manages the network adapter, broadcasting pending outgoing events and accepting incoming events
 *
 * Input: OutboundEvent, Players
 * Output: InboundEvent
 */
pub struct System {
  network: Network
}

impl System {
  pub fn new(port: u16) -> System {
    System {
      network: Network::new(port)
    }
  }
}

impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    use specs::Join;

    let (mut outbound_events, mut inbound_events, player) = arg.fetch(|w| {
      (w.write_resource::<Vec<OutboundEvent>>(),
       w.write_resource::<Vec<ClientPayload>>(),
       w.read::<PlayerAspect>())
    });

    let all_addresses = player.iter().map(|player| player.address.clone()).collect();

    // Emit all pending events
    outbound_events.drain(..)
      .flat_map(|outbound| outbound.to_server_payloads(&all_addresses))
      .foreach(|event| self.network.send(event));

    // Process all incoming events
    inbound_events.append(&mut self.network.recv_pending());
  }
}
