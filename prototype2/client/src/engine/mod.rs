mod event_handler;
use self::event_handler::EventHandler;

use std::mem;

use itertools::Itertools;
use cgmath::Quaternion;

use controller::Controller;
use renderer::Renderer;
use renderer::opengl::OpenGlRenderer;
// use client::renderer::console::ConsoleRenderer;
use protocol::InternalClientEvent;
use network::FragmentBuffer;

use common::protocol::{ClientNetworkEvent, ServerNetworkEvent, SnapshotEvent};
use common::world::ClientWorld;

pub struct Engine {
  renderer: OpenGlRenderer,
  // renderer: ConsoleRenderer,
  controller: Controller,
  events: Vec<ServerNetworkEvent>,
  partial_snapshot: FragmentBuffer,
  world: Option<ClientWorld>,
  camera_pos: (f32, f32, f32),
  camera_orient: Quaternion<f32>,
}

impl Engine {
  pub fn push_event(&mut self, event: ServerNetworkEvent) {
    self.events.push(event)
  }

  pub fn new() -> Engine {
    Engine {
      renderer: OpenGlRenderer::new(),
      controller: Controller::new(),
      events: Vec::new(),
      partial_snapshot: FragmentBuffer::None,
      world: None,
      camera_pos: (3.0, -10.0, 6.0),
      camera_orient: Quaternion::one(),
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

    internal_events.iter().foreach(|event| {
      use protocol::InternalClientEvent::CameraMove;

      match event {
        &CameraMove(ref dir) => self.on_camera_event(dir),
        _ => {},
      }
    });

    self.renderer.render_world(&self.world.as_ref(), &self.camera_pos, &self.camera_orient);

    (internal_events, outbound_events)
  }

  fn handle(&mut self, event: ServerNetworkEvent) -> Vec<ClientNetworkEvent> {
    use common::protocol::ServerNetworkEvent::*;

    match event {
      Snapshot(SnapshotEvent::PartialSnapshot(data)) => self.on_partial_snapshot(data),
      _ => Vec::new(),
    }
  }
}
