use std::mem;

use itertools::Itertools;

use client::world::ClientWorldBuffer;
use client::protocol::InternalClientEvent;

use common::world::ClientWorld;
use common::protocol::{
  ClientNetworkEvent,
  ServerNetworkEvent,
  ServerEvent
};

use client::renderer::Renderer;
use client::renderer::opengl::OpenGlRenderer;

use client::controller::Controller;

pub struct Engine {
  renderer: OpenGlRenderer,
  //console_renderer: ConsoleRenderer,
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
      controller: Controller::new(),
      events: Vec::new(),
      partial_snapshot: ClientWorldBuffer::None,
      last_snapshot: None,
      world: None,
    }
  }

  pub fn tick(&mut self) -> (Vec<InternalClientEvent>, Vec<ClientNetworkEvent>) {
    let mut event_buf = Vec::new();
    // Yank all the events off the queue, replacing with a new queue
    mem::swap(&mut self.events, &mut event_buf);

    let mut outbound_events: Vec<ClientNetworkEvent> = Vec::new();
    let mut internal_events: Vec<InternalClientEvent> = Vec::new();
    event_buf.drain(..).foreach(|event| outbound_events.append(&mut self.handle(event)));
    self.controller.handle_events(self.renderer.mut_window());
    outbound_events.append(&mut self.controller.collect_outbound_events());
    internal_events.append(&mut self.controller.collect_internal_events());

    self.renderer.render_world(&self.world.as_ref());

    (internal_events, outbound_events)
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
