use std::collections::HashMap;

use common::network;
use common::protocol::{ServerNetworkEvent};
use common::util::Newness;
use server::protocol::OutboundEvent;
use server::engine::{Engine, ClientSnapshotHistory};
use server::world::views::player::PlayerView;

// TODO: Clarify the purpose of this object
// As of right now, it handles a couple of kinds of "events" and mutates the world
// SEE: client::engine::event_handler
pub trait EventHandler {
  fn on_connect(&mut self, addr: network::Address) -> Vec<OutboundEvent>;
  fn on_disconnect(&mut self, addr: network::Address) -> Vec<OutboundEvent>;
  fn on_keep_alive(&self, addr: network::Address) -> Vec<OutboundEvent>;
  fn on_self_move(&mut self, addr: network::Address, pos: (f32, f32, f32)) -> Vec<OutboundEvent>;
  fn on_snapshot_ack(&mut self, addr: network::Address, seq_num: u16) -> Vec<OutboundEvent>;
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

  fn on_snapshot_ack(&mut self, addr: network::Address, seq_num: u16) -> Vec<OutboundEvent> {
    let player_opt = self.world.get_player_uuid_from_addr(&addr);
    match player_opt {
      Some(uuid) => {
        // NOTE: insertion should never be done here, it implies we got an ack for a snapshot
        //   that we don't know we sent
        // TODO: encode that via the type system
        let history = self.client_snapshot_histories.entry(uuid.clone()).or_insert(ClientSnapshotHistory {
          last_ack: None,
          past_snapshots: HashMap::new()
        });

        if history.last_ack.is_none() || seq_num.is_newer_than(history.last_ack.as_ref().unwrap()) {
          history.last_ack = Some(seq_num)
          // TODO: Cull old past_snapshots
        }
      },
      _ => {}
    }
    Vec::new()
  }
}
