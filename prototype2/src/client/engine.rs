use std::mem;

use itertools::Itertools;

use common::protocol::{ClientNetworkEvent, ServerNetworkEvent};

use client::renderer::Renderer;
use client::controller::Controller;

pub struct Engine {
  renderer: Renderer,
  controller: Controller,
  events: Vec<ServerNetworkEvent>,
}

impl Engine {
  pub fn push_event(&mut self, event: ServerNetworkEvent) { self.events.push(event) }

  pub fn new() -> Engine {
    Engine {
      renderer: Renderer::new(),
      controller: Controller::new(),
      events: Vec::new()
    }
  }

  pub fn tick(&mut self) -> Vec<ClientNetworkEvent> {
    let mut event_buf = Vec::new();
    // Yank all the events off the queue, replacing with a new queue
    mem::swap(&mut self.events, &mut event_buf);

    let mut outbound: Vec<ClientNetworkEvent> = Vec::new();

    event_buf.drain(..).foreach(|event| outbound.append(&mut self.handle(event)));

    outbound
  }

  fn handle(&mut self, event: ServerNetworkEvent) -> Vec<ClientNetworkEvent> {
    use common::protocol::ServerNetworkEvent::*;
    match event {
      KeepAlive => {
        println!("Server acknowledged our keepalive");
        Vec::new()
      },
      _ => Vec::new()
    }
  }
}
