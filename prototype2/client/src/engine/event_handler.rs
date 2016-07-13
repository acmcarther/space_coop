use engine::Engine;
use network::Defragmentable;
use protocol::CameraDir;
use common::protocol::{ClientNetworkEvent, StateFragment};
use common::world::ClientWorld;

// TODO: Clarify the purpose of this object
// As of right now, it handles a couple of kinds of "events" and mutates the world
// SEE: client::engine::event_handler
pub trait EventHandler  {
  fn on_partial_snapshot(&mut self, data: StateFragment) -> Vec<ClientNetworkEvent>;
  fn on_camera_event(&mut self, dir: &CameraDir);
}

impl EventHandler for Engine {
  fn on_partial_snapshot(&mut self, data: StateFragment) -> Vec<ClientNetworkEvent> {
    self.partial_snapshot.integrate(data);
    let world_opt = ClientWorld::defragment(&self.partial_snapshot);
    if world_opt.is_some() {
      let (seq_num, world) = world_opt.unwrap();
      self.world = Some(world);
      vec![ClientNetworkEvent::SnapshotAck(seq_num)]
    } else {
      Vec::new()
    }
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
