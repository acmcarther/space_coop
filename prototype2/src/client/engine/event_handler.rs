use client::engine::Engine;
use client::network::Defragmentable;
use client::protocol::CameraDir;
use common::protocol::{ClientNetworkEvent, StateFragment};
use common::world::ClientWorld;

pub trait EventHandler  {
  fn on_partial_snapshot(&mut self, data: StateFragment) -> Vec<ClientNetworkEvent>;
  fn on_camera_event(&mut self, dir: &CameraDir);
}

impl EventHandler for Engine {
  fn on_partial_snapshot(&mut self, data: StateFragment) -> Vec<ClientNetworkEvent> {
    self.partial_snapshot.integrate(data);
    let world_opt = ClientWorld::defragment(&self.partial_snapshot);
    if world_opt.is_some() { self.world = world_opt; }
    Vec::new()
  }

  fn on_camera_event(&mut self, dir: &CameraDir) {
    match dir {
      &CameraDir::Forward  => self.camera_pos.0 = self.camera_pos.0 + 0.1,
      &CameraDir::Backward => self.camera_pos.0 = self.camera_pos.0 - 0.1,
      &CameraDir::Left     => self.camera_pos.1 = self.camera_pos.1 - 0.1,
      &CameraDir::Right    => self.camera_pos.1 = self.camera_pos.1 - 0.1,
    }
  }
}
