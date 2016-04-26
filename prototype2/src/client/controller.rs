use std::mem;

use common::protocol::{
  ClientNetworkEvent,
  ClientEvent
};

use client::protocol::InternalClientEvent;

use itertools::Itertools;

use glutin;

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
    match event {
      &glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) => {
        self.internal_events.push(InternalClientEvent::Exit)
      },
      _ => {}
    }
  }

  fn handle_outbound(&mut self, event: &glutin::Event) {
    match event {
      &glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::W)) => {
        self.outbound_events.push(ClientNetworkEvent::DomainEvent(ClientEvent::SelfMove { x_d: 0.1, y_d: 0.0, z_d: 0.0 }))
      },
      &glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::A)) => {
        self.outbound_events.push(ClientNetworkEvent::DomainEvent(ClientEvent::SelfMove { x_d: 0.0, y_d: 0.1, z_d: 0.0 }))
      },
      &glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::S)) => {
        self.outbound_events.push(ClientNetworkEvent::DomainEvent(ClientEvent::SelfMove { x_d: -0.1, y_d: 0.0, z_d: 0.0 }))
      },
      &glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::D)) => {
        self.outbound_events.push(ClientNetworkEvent::DomainEvent(ClientEvent::SelfMove { x_d: 0.0, y_d: -0.1, z_d: 0.0 }))
      },
      &glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Q)) => {
        self.outbound_events.push(ClientNetworkEvent::DomainEvent(ClientEvent::SelfMove { x_d: 0.0, y_d: 0.0, z_d: 0.1 }))
      },
      &glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::E)) => {
        self.outbound_events.push(ClientNetworkEvent::DomainEvent(ClientEvent::SelfMove { x_d: 0.0, y_d: 0.0, z_d: -0.1 }))
      },
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
