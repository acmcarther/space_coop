mod event_handler;
use self::event_handler::EventHandler;

use std::mem;
use std::collections::HashMap;

use time::{self, Duration};
use itertools::Itertools;
use uuid::Uuid;

use common::protocol::{
  ClientEvent,
  ClientPayload,
  ServerPayload,
};
use common::world::ClientWorld;
use server::protocol::OutboundEvent;
use server::world::ServerWorld;
use server::world::views::player::PlayerView;
use server::world::views::client_world::ClientWorldView;
use server::network::Fragmentable;
use server::physics::Physics;

pub struct ClientSnapshotHistory {
  pub last_ack: Option<u16>,
  pub past_snapshots: HashMap<u16, ClientWorld>
}

pub struct Engine {
  // TODO: this is pub so Physics can manipulate it
  //   add an analogue to View so it doesn't need to be pub
  pub world: ServerWorld,
  events: Vec<ClientPayload>,
  snapshot_idx: u16,
  client_snapshot_histories: HashMap<Uuid, ClientSnapshotHistory>
}

impl Engine {
  pub fn push_event(&mut self, event: ClientPayload) { self.events.push(event) }

  pub fn new() -> Engine {
    Engine {
      world: ServerWorld::new(),
      events: Vec::new(),
      snapshot_idx: 0,
      client_snapshot_histories: HashMap::new(),
    }
  }

  pub fn tick(&mut self, dt: &time::Duration) -> Vec<ServerPayload> {
    let mut event_buf = Vec::new();
    // Yank all the events off the queue, replacing with a new queue
    mem::swap(&mut self.events, &mut event_buf);

    let mut outbound: Vec<OutboundEvent> = Vec::new();


    event_buf.drain(..).foreach(|event| outbound.append(&mut self.handle(event)));

    self.tick_physics(dt);

    self.validate_connections();

    self.snapshot_idx = self.snapshot_idx.wrapping_add(1);

    // TODO: Clean up borrowck nonsense here
    outbound.extend(self.world.player.iter()
      .filter(|&(_, ply)| ply.connected)
      .map(|(uuid, ply)| (uuid.clone(), ply.address.clone()))
      .collect::<Vec<_>>().into_iter() // Dodge borrow checker
      .flat_map(|(uuid, addr)| {
        let addr = addr.clone();
        self.world.as_client_world(&uuid).fragment_to_events(self.snapshot_idx)
          .into_iter().map(|partial| (addr.clone(), partial))
          .collect::<Vec<_>>().into_iter() // Dodging borrow checker again
      })
      .map(|(addr, event)| OutboundEvent::Directed{dest: addr, event: event})
    );

    // TODO: optimize this iter usage, its inefficient because of the transformations
    //   to vector and back
    let all_addrs = self.world.all_connected_addrs();
    outbound.into_iter()
      .flat_map(|outbound| outbound.to_server_payloads(&all_addrs))
      .collect::<Vec<ServerPayload>>()
  }

  fn handle(&mut self, payload: ClientPayload) -> Vec<OutboundEvent> {
    use common::protocol::ClientNetworkEvent::*;

    if let Some(ply_uuid) = self.world.addr_to_player.get(&payload.address).map(|a| a.clone()) {
      self.world.update_player_last_msg(&ply_uuid);
    }

    match payload.event {
      Connect => self.on_connect(payload.address),
      Disconnect => self.on_disconnect(payload.address),
      KeepAlive => self.on_keep_alive(payload.address),
      SnapshotAck(seq_num) => self.on_snapshot_ack(payload.address, seq_num),
      DomainEvent(ClientEvent::SelfMove {x_d, y_d, z_d}) => self.on_self_move(payload.address, (x_d, y_d, z_d))
    }
  }

  fn validate_connections(&mut self) -> Vec<OutboundEvent> {
    let timeout = Duration::seconds(5);
    let now = time::now();

    self.world.player.values()
      .filter(|ply| ply.connected && timeout < now - ply.last_msg)
      .map(|ply| ply.address.clone())
      .collect::<Vec<_>>()   // Resolve borrow issue for subsequent flatmap
      .into_iter()
      .flat_map(|addr| self.on_disconnect(addr))
      .collect()
  }
}
