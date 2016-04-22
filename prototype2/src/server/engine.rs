use std::collections::HashMap;
use std::mem;

use uuid::{
  Uuid, UuidVersion
};

use serde_json;

use common::protocol::{
  ClientEvent,
  ClientNetworkEvent,
  ClientPayload,
  ServerPayload,
  ServerEvent,
  ServerNetworkEvent,
};
use server::protocol::{
  OutboundEvent,
};
use server::world::ServerWorld;
use itertools::Itertools;

pub struct Engine {
  world: ServerWorld,
  events: Vec<ClientPayload>,
  snapshot_idx: u32,
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
    let mut snapshot_bytes = client_snapshot.into_bytes();
    let mut snapshot_byte_sets = snapshot_bytes.chunks(512 /*bytes*/).enumerate();
    let set_count = snapshot_byte_sets.len();
    self.snapshot_idx = self.snapshot_idx.wrapping_add(1);

    outbound.append(&mut snapshot_byte_sets.map(|(idx, bytes)| {
      OutboundEvent::Undirected(ServerNetworkEvent::DomainEvent(ServerEvent::FullSnapshot {
        series: self.snapshot_idx,
        idx: idx as u32,
        count: set_count as u32,
        state_fragment: bytes.to_vec()
      }))
    }).collect::<Vec<OutboundEvent>>());

    // TODO: optimize this iter usage, its inefficient because of the trnaformations
    //   to vector and back
    outbound.into_iter()
      .flat_map(|outbound| outbound.to_server_payloads(&|uuid| { self.world.get_player_addr_from_uuid(&uuid).map(|addr| addr.clone())}))
      .collect::<Vec<ServerPayload>>()
  }


  fn handle(&mut self, payload: ClientPayload) -> Vec<OutboundEvent> {
    use common::protocol::ClientNetworkEvent::*;

    let addr = payload.address;
    match payload.event {
      Connect => {
        println!("A person connected: {}", addr);
        let player_uuid = self.world.get_player_uuid_from_addr(&addr)
          .map(|v| v.clone())
          .unwrap_or_else(|| self.world.add_player(addr));

        let player = self.world.get_mut_player(&player_uuid).unwrap(); // Safe, because of above insertion
        player.set_connected(true);
        vec![OutboundEvent::External{dest: addr, event: ServerNetworkEvent::Connected}]
      },
      Disconnect => {
        println!("A person disconnected: {}", addr);
        let player_opt = self.world.get_player_uuid_from_addr(&addr).map(|v| v.clone())
          .and_then(|uuid| self.world.get_mut_player(&uuid));
        match player_opt {
          None => vec![OutboundEvent::External{dest: addr, event: ServerNetworkEvent::Error("Tried to disconnect, but not connected to server".to_owned())}],
          Some(player) => {
            player.set_connected(false);
            vec![OutboundEvent::External{dest: addr, event: ServerNetworkEvent::Disconnected}]
          }
        }
      },
      KeepAlive => {
        println!("{} is keeping alive", addr);
        let player_opt = self.world.get_player_uuid_from_addr(&addr)
          .and_then(|uuid| self.world.get_player(uuid));
        match player_opt {
          Some(player) => vec![OutboundEvent::Directed{dest: player.uuid().clone(), event: ServerNetworkEvent::KeepAlive}],
          _ => Vec::new()
        }
      },
      DomainEvent(ClientEvent::SelfMove {x_d, y_d, z_d}) => Vec::new(),
    }
  }
}
