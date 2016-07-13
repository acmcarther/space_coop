use uuid::Uuid;

use common::world::ClientWorld;
use world::WorldContainer;

pub trait ClientWorldView {
  fn as_client_world(&self, ply_uuid: &Uuid) -> ClientWorld;
}

impl <T: WorldContainer> ClientWorldView for T {
  fn as_client_world(&self, ply_uuid: &Uuid) -> ClientWorld {
    ClientWorld {
      own_entity: self.world().controller.get(ply_uuid).map(|e| e.subject.clone()),
      entities: self.world().entities.clone(),
      rendered: self.world().rendered.clone(),
      physical: self.world().physical.clone(),
      disabled: self.world().disabled.clone(),
    }
  }
}
