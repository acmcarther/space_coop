use common::network;
use common::protocol::{ServerNetworkEvent};
use server::protocol::OutboundEvent;
use server::engine::Engine;
use server::world::views::player::PlayerView;

pub trait EventHandler {
  fn on_connect(&mut self, addr: network::Address) -> Vec<OutboundEvent>;
  fn on_disconnect(&mut self, addr: network::Address) -> Vec<OutboundEvent>;
  fn on_keep_alive(&self, addr: network::Address) -> Vec<OutboundEvent>;
  fn on_self_move(&mut self, addr: network::Address, pos: (f32, f32, f32)) -> Vec<OutboundEvent>;
  fn on_snapshot_ack(&mut self, _: network::Address) -> Vec<OutboundEvent>;
}

impl EventHandler for Engine {
  fn on_connect(&mut self, addr: network::Address) -> Vec<OutboundEvent> {
    self.world.player_connect(addr);
    vec![OutboundEvent::Directed{dest: addr, event: ServerNetworkEvent::Connected}]
  }

  fn on_disconnect(&mut self, addr: network::Address) -> Vec<OutboundEvent> {
    let disconnected = self.world.player_disconnect(&addr);
    if disconnected {
      vec![OutboundEvent::Directed{dest: addr, event: ServerNetworkEvent::Disconnected}]
    } else {
      vec![OutboundEvent::Directed{dest: addr, event: ServerNetworkEvent::Error("Tried to disconnect, but not connected to server".to_owned())}]
    }
  }

  fn on_keep_alive(&self, addr: network::Address) -> Vec<OutboundEvent> {
    let player_opt = self.world.get_player_uuid_from_addr(&addr);
    match player_opt {
      Some(_) => vec![OutboundEvent::Directed{dest: addr, event: ServerNetworkEvent::KeepAlive}],
      _ => Vec::new()
    }
  }


  fn on_self_move(&mut self, addr: network::Address, pos: (f32, f32, f32)) -> Vec<OutboundEvent> {
    let (x_d, y_d, z_d) = pos;
    if let Some(uuid) = self.world.get_player_uuid_from_addr(&addr).map(|v| v.clone()) {
      self.world.move_player_ent(&uuid, x_d, y_d, z_d)
    }
    Vec::new()
  }

  fn on_snapshot_ack(&mut self, _: network::Address) -> Vec<OutboundEvent> {
    Vec::new()
  }
}
