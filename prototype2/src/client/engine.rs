use std::mem;

use itertools::Itertools;

use client::world::ClientWorldBuffer;

use common::world::ClientWorld;
use common::protocol::{
  ClientNetworkEvent,
  ServerNetworkEvent,
  ServerEvent
};

use client::renderer::Renderer;
use client::renderer::opengl::OpenGlRenderer;
use client::renderer::console::ConsoleRenderer;

use client::controller::Controller;

pub struct Engine {
  renderer: OpenGlRenderer,
  console_renderer: ConsoleRenderer,
  controller: Controller,
  events: Vec<ServerNetworkEvent>,
  partial_snapshot: ClientWorldBuffer,
  last_snapshot: Option<u16>,
  world: Option<ClientWorld>
}

impl Engine {
  pub fn push_event(&mut self, event: ServerNetworkEvent) { self.events.push(event) }

  pub fn new() -> Engine {
    Engine {
      renderer: OpenGlRenderer::new(),
      console_renderer: ConsoleRenderer::new(),
      controller: Controller::new(),
      events: Vec::new(),
      partial_snapshot: ClientWorldBuffer::None,
      last_snapshot: None,
      world: None,
    }
  }

  pub fn tick(&mut self) -> Vec<ClientNetworkEvent> {
    let mut event_buf = Vec::new();
    // Yank all the events off the queue, replacing with a new queue
    mem::swap(&mut self.events, &mut event_buf);

    let mut outbound: Vec<ClientNetworkEvent> = Vec::new();
    event_buf.drain(..).foreach(|event| outbound.append(&mut self.handle(event)));
    outbound.append(&mut self.controller.recv_pending_net(self.renderer.mut_window()));

    self.renderer.render_world(&self.world.as_ref());
    //self.console_renderer.render_world(&self.world.as_ref());

    outbound
  }

  fn handle(&mut self, event: ServerNetworkEvent) -> Vec<ClientNetworkEvent> {
    use common::protocol::ServerNetworkEvent::*;

    match event {
      DomainEvent(ServerEvent::PartialSnapshot(data)) => {
        self.partial_snapshot.integrate(data);
        let world_opt = self.partial_snapshot.try_collate();
        if world_opt.is_some() { self.world = world_opt; }
        Vec::new()
      },
      _ => Vec::new()
    }
  }

  pub fn other_snapshot_newer(&self, series: u16) -> bool {
    match self.last_snapshot {
      None => true,
      Some(u) => {
        let pos_diff = series.wrapping_sub(u);
        pos_diff != 0 && pos_diff < 32000
      }
    }
  }
}
