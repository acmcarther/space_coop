use common::protocol::{
  ClientNetworkEvent,
  ClientEvent
};

use glutin;

pub struct Controller;

impl Controller {
  pub fn new() -> Controller {
    Controller
  }

  pub fn recv_pending_net(&mut self, window: &mut glutin::Window) -> Vec<ClientNetworkEvent> {
    let mut buf = Vec::new();
    for event in window.poll_events() {
      match event {
        glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::W)) => {
          buf.push(ClientNetworkEvent::DomainEvent(ClientEvent::SelfMove { x_d: 0.1, y_d: 0.0, z_d: 0.0 }))
        },
        glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::A)) => {
          buf.push(ClientNetworkEvent::DomainEvent(ClientEvent::SelfMove { x_d: 0.0, y_d: 0.1, z_d: 0.0 }))
        },
        glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::S)) => {
          buf.push(ClientNetworkEvent::DomainEvent(ClientEvent::SelfMove { x_d: -0.1, y_d: 0.0, z_d: 0.0 }))
        },
        glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::D)) => {
          buf.push(ClientNetworkEvent::DomainEvent(ClientEvent::SelfMove { x_d: 0.0, y_d: -0.1, z_d: 0.0 }))
        },
        glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Q)) => {
          buf.push(ClientNetworkEvent::DomainEvent(ClientEvent::SelfMove { x_d: 0.0, y_d: 0.0, z_d: 0.1 }))
        },
        glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::E)) => {
          buf.push(ClientNetworkEvent::DomainEvent(ClientEvent::SelfMove { x_d: 0.0, y_d: 0.0, z_d: -0.1 }))
        },
        _ => {},
      }
    }
    buf
  }
}
