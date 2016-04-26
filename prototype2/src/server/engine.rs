use std::mem;

use serde_json;

use common::protocol::{
  ClientEvent,
  ClientPayload,
  ServerPayload,
  ServerEvent,
  ServerNetworkEvent,
  PartialClientSnapshot
};
use server::protocol::{
  OutboundEvent,
};

use common::network;

use server::world::ServerWorld;
use server::world::views::player::PlayerView;
use itertools::Itertools;

use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

pub struct Engine {
  world: ServerWorld,
  events: Vec<ClientPayload>,
  snapshot_idx: u16,
}

impl Engine {
  pub fn push_event(&mut self, event: ClientPayload) { self.events.push(event) }

  pub fn new() -> Engine {
    Engine {
      world: ServerWorld::new(),
      events: Vec::new(),
      snapshot_idx: 0
    }
  }

  pub fn tick(&mut self) -> Vec<ServerPayload> {
    let mut event_buf = Vec::new();
    // Yank all the events off the queue, replacing with a new queue
    mem::swap(&mut self.events, &mut event_buf);

    let mut outbound: Vec<OutboundEvent> = Vec::new();

    event_buf.drain(..).foreach(|event| outbound.append(&mut self.handle(event)));

    let client_snapshot = serde_json::to_string(&self.world.as_client_world()).unwrap();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::Default);
    encoder.write(client_snapshot.as_bytes());
    let snapshot_bytes = encoder.finish().unwrap(); // Assumed to be safe because I control the format
    println!("snapshot_bytes_len:{}", snapshot_bytes.len());
    let snapshot_byte_sets = snapshot_bytes.chunks(128 /*bytes*/).enumerate();
    let set_count = snapshot_byte_sets.len();
    println!("sets: {}", set_count);
    self.snapshot_idx = self.snapshot_idx.wrapping_add(1);

    outbound.append(&mut snapshot_byte_sets.map(|(idx, bytes)| {
      OutboundEvent::Undirected(ServerNetworkEvent::DomainEvent(ServerEvent::PartialSnapshot(PartialClientSnapshot {
        series: self.snapshot_idx,
        idx: idx as u32,
        count: set_count as u32,
        state_fragment: bytes.to_vec()
      })))
    }).collect::<Vec<OutboundEvent>>());

    // TODO: optimize this iter usage, its inefficient because of the transformations
    //   to vector and back
    let all_addrs = self.world.all_connected_addrs();
    outbound.into_iter()
      .flat_map(|outbound| outbound.to_server_payloads(&all_addrs, &|uuid| { self.world.get_player_addr_from_uuid(&uuid).map(|addr| addr.clone())}))
      .collect::<Vec<ServerPayload>>()
  }

  fn handle(&mut self, payload: ClientPayload) -> Vec<OutboundEvent> {
    use common::protocol::ClientNetworkEvent::*;

    match payload.event {
      Connect => self.on_connect(payload.address),
      Disconnect => self.on_disconnect(payload.address),
      KeepAlive => self.on_keep_alive(payload.address),
      DomainEvent(ClientEvent::SelfMove {x_d, y_d, z_d}) => self.on_self_move(payload.address, (x_d, y_d, z_d))
    }
  }

  fn on_connect(&mut self, addr: network::Address) -> Vec<OutboundEvent> {
    self.world.player_connect(addr);
    vec![OutboundEvent::External{dest: addr, event: ServerNetworkEvent::Connected}]
  }

  fn on_disconnect(&mut self, addr: network::Address) -> Vec<OutboundEvent> {
    let disconnected = self.world.player_disconnect(&addr);
    if disconnected {
      vec![OutboundEvent::External{dest: addr, event: ServerNetworkEvent::Disconnected}]
    } else {
      vec![OutboundEvent::External{dest: addr, event: ServerNetworkEvent::Error("Tried to disconnect, but not connected to server".to_owned())}]
    }
  }

  fn on_keep_alive(&self, addr: network::Address) -> Vec<OutboundEvent> {
    let player_opt = self.world.get_player_uuid_from_addr(&addr);
    match player_opt {
      Some(uuid) => vec![OutboundEvent::Directed{dest: uuid.clone(), event: ServerNetworkEvent::KeepAlive}],
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
}
