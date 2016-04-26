use uuid::Uuid;

use common::network;
use common::world::{RenderAspect, PhysicalAspect};
use server::world::{ControllerAspect, PlayerAspect, WorldContainer};

pub trait PlayerView {
  fn player_connect(&mut self, address: network::Address);
  fn player_disconnect(&mut self, address: &network::Address) -> bool;
  fn all_connected_addrs(&self) -> Vec<network::Address>;
  fn get_player_uuid_from_addr(&self, address: &network::Address) -> Option<&Uuid>;
  fn get_player_addr_from_uuid(&self, uuid: &Uuid) -> Option<&network::Address>;
  fn move_player_ent(&mut self, player: &Uuid, x_d: f32, y_d: f32, z_d: f32);
}

impl <T: WorldContainer> PlayerView for T {
  fn player_connect(&mut self, address: network::Address) {
    let player_uuid_opt = self.world().addr_to_player.get(&address).map(|v| v.clone());
    match player_uuid_opt {
      Some(player_uuid) => {
        if self.mut_world().player.contains_key(&player_uuid) {
          self.mut_world().player.get_mut(&player_uuid).unwrap().connected = true;
          if let Some(subject_uuid) = self.mut_world().controller.get(&player_uuid).map(|ctrl| ctrl.subject.clone()) {
            self.mut_world().disabled.remove(&subject_uuid);
          }
        } else {
          // TODO: ADDR_TO_PLAYER DESYNC
          // TODO: Data structure should not permit this outcome
        }
      }
      None => {
        let player_uuid = Uuid::new_v4();
        let player_subject_uuid = Uuid::new_v4();
        self.mut_world().entities.push(player_uuid.clone());
        self.mut_world().entities.push(player_subject_uuid.clone());
        self.mut_world().rendered.insert(player_subject_uuid.clone(), RenderAspect::new());
        self.mut_world().physical.insert(player_subject_uuid.clone(), PhysicalAspect::new((0.0,0.0,0.0), false));
        self.mut_world().player.insert(player_uuid.clone(), PlayerAspect::new(address.clone(), true));
        self.mut_world().controller.insert(player_uuid.clone(), ControllerAspect::new(player_subject_uuid.clone()));
        self.mut_world().addr_to_player.insert(address, player_uuid.clone());
      }
    }
  }

  fn player_disconnect(&mut self, address: &network::Address) -> bool {
    match self.world().addr_to_player.get(address).map(|v| v.clone()) {
      Some(player_uuid) => {
        if self.mut_world().player.contains_key(&player_uuid) {
          self.mut_world().player.get_mut(&player_uuid).unwrap().connected = false;
          if let Some(subject_uuid) = self.mut_world().controller.get(&player_uuid).map(|ctrl| ctrl.subject.clone()) {
            self.mut_world().disabled.insert(subject_uuid);
          }
          true
        } else {
          // TODO: ADDR_TO_PLAYER DESYNC
          // TODO: Data structure should not permit this outcome
          false
        }
      },
      None => false
    }
  }

  fn all_connected_addrs(&self) -> Vec<network::Address> {
    self.world().player.values()
      .filter(|p| p.connected)
      .map(|p| p.address.clone())
      .collect()
  }

  fn get_player_uuid_from_addr(&self, address: &network::Address) -> Option<&Uuid> {
    self.world().addr_to_player.get(address)
  }

  fn get_player_addr_from_uuid(&self, uuid: &Uuid) -> Option<&network::Address> {
    self.world().player.get(uuid).map(|p| &p.address)
  }

  // TODO: this goes in a different view
  fn move_player_ent(&mut self, uuid: &Uuid, x_d: f32, y_d: f32, z_d: f32) {
    let subject = self.world().controller.get(uuid).map(|a| a.subject.clone());
    if subject.is_none() { return; }
    match self.mut_world().physical.get_mut(&subject.unwrap()) {
      Some(aspect) => {
        aspect.pos.0 = aspect.pos.0 + x_d;
        aspect.pos.1 = aspect.pos.1 + y_d;
        aspect.pos.2 = aspect.pos.2 + z_d;
      },
      None => {}
    }
  }
}
