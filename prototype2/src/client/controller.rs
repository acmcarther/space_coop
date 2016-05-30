use std::mem;

use glutin;
use itertools::Itertools;

use client::protocol::{InternalClientEvent, CameraDir};
use common::protocol::{ClientNetworkEvent};

/**
 * An event generator for "internal" and "external" events
 *
 * TODO: Abstract this into an interface to be implemented by types associated to renderers.
 * This implementation is actually specific to the OpenGL renderer. A console render should
 * be able to emit events as well.
 */
pub struct Controller {
  internal_events: Vec<InternalClientEvent>,
  outbound_events: Vec<ClientNetworkEvent>,
}

impl Controller {
  pub fn new() -> Controller {
    Controller {
      internal_events: Vec::new(),
      outbound_events: Vec::new(),
    }
  }

  pub fn handle_events(&mut self, window: &mut glutin::Window) {
    window.poll_events().foreach(|event| {
      self.handle_internal(&event);
      self.handle_outbound(&event);
    });
  }

  fn handle_internal(&mut self, event: &glutin::Event) {
    use glutin::Event::*;
    use glutin::VirtualKeyCode::*;
    use client::protocol::InternalClientEvent::{CameraMove, Exit};

    match event {
      &KeyboardInput(_, _, Some(Escape)) => self.internal_events.push(Exit),
      &KeyboardInput(_, _, Some(I)) => self.internal_events.push(CameraMove(CameraDir::Forward)),
      &KeyboardInput(_, _, Some(J)) => self.internal_events.push(CameraMove(CameraDir::Left)),
      &KeyboardInput(_, _, Some(K)) => self.internal_events.push(CameraMove(CameraDir::Backward)),
      &KeyboardInput(_, _, Some(L)) => self.internal_events.push(CameraMove(CameraDir::Right)),
      _ => {}
    }
  }

  fn handle_outbound(&mut self, event: &glutin::Event) {
    use glutin::Event::*;
    use glutin::VirtualKeyCode::*;
    use common::protocol::ClientNetworkEvent::DomainEvent;
    use common::protocol::ClientEvent::SelfMove;

    match event {
      &KeyboardInput(_, _, Some(W)) => self.outbound_events.push(DomainEvent(SelfMove { x_d: 0.1, y_d: 0.0, z_d: 0.0 })),
      &KeyboardInput(_, _, Some(A)) => self.outbound_events.push(DomainEvent(SelfMove { x_d: 0.0, y_d: 0.1, z_d: 0.0 })),
      &KeyboardInput(_, _, Some(S)) => self.outbound_events.push(DomainEvent(SelfMove { x_d: -0.1, y_d: 0.0, z_d: 0.0 })),
      &KeyboardInput(_, _, Some(D)) => self.outbound_events.push(DomainEvent(SelfMove { x_d: 0.0, y_d: -0.1, z_d: 0.0 })),
      &KeyboardInput(_, _, Some(Q)) => self.outbound_events.push(DomainEvent(SelfMove { x_d: 0.0, y_d: 0.0, z_d: 0.1 })),
      &KeyboardInput(_, _, Some(E)) => self.outbound_events.push(DomainEvent(SelfMove { x_d: 0.0, y_d: 0.0, z_d: -0.1 })),
      _ => {},
    }
  }

  pub fn collect_internal_events(&mut self) -> Vec<InternalClientEvent> {
    let mut output = Vec::new();
    mem::swap(&mut self.internal_events, &mut output);
    output
  }

  pub fn collect_outbound_events(&mut self) -> Vec<ClientNetworkEvent> {
    let mut output = Vec::new();
    mem::swap(&mut self.outbound_events, &mut output);
    output
  }
}
