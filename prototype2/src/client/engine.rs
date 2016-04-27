use std::mem;

use itertools::Itertools;
use cgmath::Quaternion;

use client::controller::Controller;
use client::renderer::Renderer;
use client::renderer::opengl::OpenGlRenderer;
use client::protocol::{InternalClientEvent, CameraDir};
use client::network::FragmentBuffer;
use client::network::Defragmentable;

use common::protocol::{ClientNetworkEvent, ServerNetworkEvent, SnapshotEvent};
use common::world::ClientWorld;

pub struct Engine {
  renderer: OpenGlRenderer,
  //console_renderer: ConsoleRenderer,
  controller: Controller,
  events: Vec<ServerNetworkEvent>,
  partial_snapshot: FragmentBuffer,
  last_snapshot: Option<u16>,
  world: Option<ClientWorld>,
  camera_pos: (f32, f32, f32),
  camera_orient: Quaternion<f32>
}

impl Engine {
  pub fn push_event(&mut self, event: ServerNetworkEvent) { self.events.push(event) }

  pub fn new() -> Engine {
    Engine {
      renderer: OpenGlRenderer::new(),
      controller: Controller::new(),
      events: Vec::new(),
      partial_snapshot: FragmentBuffer::None,
      last_snapshot: None,
      world: None,
      camera_pos: (1.5, -5.0, 3.0),
      camera_orient: Quaternion::one()
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
      use client::protocol::InternalClientEvent::CameraMove;

      match event {
        &CameraMove(CameraDir::Forward)  => self.camera_pos.0 = self.camera_pos.0 + 0.1,
        &CameraMove(CameraDir::Backward) => self.camera_pos.0 = self.camera_pos.0 - 0.1,
        &CameraMove(CameraDir::Left)     => self.camera_pos.1 = self.camera_pos.1 - 0.1,
        &CameraMove(CameraDir::Right)    => self.camera_pos.1 = self.camera_pos.1 - 0.1,
        _ => {}
      }
    });

    self.renderer.render_world(&self.world.as_ref(), &self.camera_pos, &self.camera_orient);

    (internal_events, outbound_events)
  }

  fn handle(&mut self, event: ServerNetworkEvent) -> Vec<ClientNetworkEvent> {
    use common::protocol::ServerNetworkEvent::*;

    match event {
      Snapshot(SnapshotEvent::PartialSnapshot(data)) => {
        self.partial_snapshot.integrate(data);
        let world_opt = ClientWorld::defragment(&self.partial_snapshot);
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
