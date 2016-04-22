use common::protocol::ClientNetworkEvent;
use client::protocol::ClientLocalEvent;

pub struct Controller;

impl Controller {
  pub fn new() -> Controller {
    Controller
  }

  pub fn recv_pending_net(&mut self) -> Vec<ClientNetworkEvent> {
    Vec::new()
  }

  pub fn recv_pending_local(&mut self) -> Vec<ClientLocalEvent> {
    Vec::new()
  }
}
