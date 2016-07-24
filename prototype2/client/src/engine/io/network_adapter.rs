use specs;
use engine;

use network::Network;
use std::net::SocketAddr;
use common::protocol::{ClientNetworkEvent, ServerNetworkEvent};
use std::sync::mpsc::Receiver;

use itertools::Itertools;

/**
 * Manages the network adapter, broadcasting pending outgoing events and accepting incoming events
 *
 * Also handles telling the server we're disconnecting.
 *
 * Input: ServerNetworkEvent,
 * Output: ClientNetworkEvent
 */
pub struct System {
  network: Network,
  network_kill_signal: Receiver<()>,
}

impl System {
  pub fn new(port: u16, server_addr: SocketAddr, network_kill_signal: Receiver<()>) -> System {
    let mut network = Network::new(port, server_addr);
    let success = network.connect();
    if !success {
      println!("Could not connect to {}", server_addr.to_string());
      panic!("could not connect, i need to make this not out of band");
    }
    System {
      network: network,
      network_kill_signal: network_kill_signal,
    }
  }
}

impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    let (mut outbound_events, mut inbound_events) = arg.fetch(|w| {
      (w.write_resource::<Vec<ClientNetworkEvent>>(), w.write_resource::<Vec<ServerNetworkEvent>>())
    });

    outbound_events.drain(..)
      .foreach(|event| self.network.send(event));

    inbound_events.append(&mut self.network.recv_pending());

    if let Ok(()) = self.network_kill_signal.try_recv() {
      self.network.disconnect();
    }
  }
}
