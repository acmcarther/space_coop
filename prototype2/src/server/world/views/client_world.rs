use common::world::ClientWorld;
use server::world::WorldContainer;

pub trait ClientWorldView {
  fn as_client_world(&self) -> ClientWorld;
}

impl <T: WorldContainer> ClientWorldView for T {
  fn as_client_world(&self) -> ClientWorld {
    ClientWorld {
      entities: self.world().entities.clone(),
      rendered: self.world().rendered.clone(),
      physical: self.world().physical.clone(),
      disabled: self.world().disabled.clone(),
    }
  }
}
