use server::world::{
  ServerWorld,
  Player
};
use common::network;
use uuid::Uuid;

use common::world::{
  Entity,
  RenderAspect,
  ClientWorld,
  PhysicalAspect,
};
use server::world::ControlledAspect;

pub struct PlayerView<'a> {
  world: &'a mut ServerWorld
}

impl <'a> PlayerView<'a> {
  pub fn new(world: &'a mut ServerWorld) -> PlayerView<'a> {
    PlayerView { world: world }
  }

  // TODO: Dedupe /////////
  pub fn get_player_uuid_from_addr(&self, address: &network::Address) -> Option<&Uuid> {
    self.world.addr_to_player.get(address)
  }

  pub fn get_player_addr_from_uuid(&self, uuid: &Uuid) -> Option<&network::Address> {
    self.get_player(uuid).map(|p| p.address())
  }

  pub fn get_player(&self, uuid: &Uuid) -> Option<&Player> {
    self.world.players.get(uuid)
  }

  pub fn get_mut_player(&mut self, uuid: &Uuid) -> Option<&mut Player> {
    self.world.players.get_mut(uuid)
  }

  pub fn move_player_ent(&mut self, uuid: &Uuid, x_d: f32, y_d: f32, z_d: f32) {

    let uuid = self.world.controlled_inverse.get(uuid).map(|a| a.uuid.clone());

    if uuid.is_none() { return; }

    match self.world.physical.get_mut(&uuid.unwrap()) {
      Some(aspect) => {
        aspect.pos.0 = aspect.pos.0 + x_d;
        aspect.pos.1 = aspect.pos.1 + y_d;
        aspect.pos.2 = aspect.pos.2 + z_d;
      },
      None => {}
    }
  }

  pub fn add_player(&mut self, addr: network::Address) -> Uuid {
    let player = Player::new(addr.clone());
    let player_ent_uuid = Uuid::new_v4();
    self.world.entities.push(player_ent_uuid.clone());
    self.world.rendered.insert(player_ent_uuid.clone(), RenderAspect::new());
    self.world.physical.insert(player_ent_uuid.clone(), PhysicalAspect::new((0.0,0.0,0.0), false));
    self.world.controlled.insert(player_ent_uuid.clone(), ControlledAspect::new(player.uuid().clone()));
    self.world.controlled_inverse.insert(player.uuid().clone(), ControlledAspect::new(player_ent_uuid.clone()));

    let uuid = player.uuid().clone();
    self.world.addr_to_player.insert(addr, uuid.clone());
    self.world.players.insert(uuid.clone(), player);
    uuid
  }
}
