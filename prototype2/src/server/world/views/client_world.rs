use uuid::Uuid;

use common::world::ClientWorld;
use server::world::WorldContainer;

pub trait ClientWorldView {
  fn as_client_world(&self, ply_uuid: &Uuid) -> ClientWorld;
}

impl <T: WorldContainer> ClientWorldView for T {
  fn as_client_world(&self, ply_uuid: &Uuid) -> ClientWorld {
    ClientWorld {
      own_entity: ply_uuid.clone(),
      entities: self.world().entities.clone(),
      rendered: self.world().rendered.clone(),
      physical: self.world().physical.clone(),
      disabled: self.world().disabled.clone(),
    }
  }
}
