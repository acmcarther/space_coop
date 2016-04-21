use time::{self, Duration, Tm};
use uuid::{Uuid, UuidVersion};

use server::network;

use std::collections::HashMap;

#[derive(Clone)]
pub struct Position {
  x: f32,
  y: f32,
  z: f32
}

impl Position {
  pub fn origin() -> Position {
    Position { x: 0.0, y: 0.0, z: 0.0 }
  }
}

#[derive(Clone)]
pub struct NetStatus {
  connected: bool,
  last_msg: Tm,
  addr: network::Address,
}

impl NetStatus {
  pub fn new_connected(source: network::Address) -> NetStatus {
    NetStatus {
      connected: true,
      last_msg: time::now(),
      addr: source
    }
  }

  pub fn connected(&self) -> bool {
    self.connected
  }

  pub fn connect(&mut self) {
    self.connected = true
  }

  pub fn disconnect(&mut self) {
    self.connected = false
  }

  pub fn update_last(&mut self) {
    self.last_msg = time::now()
  }
}

#[derive(Clone)]
pub struct PlayerState {
  uuid: Uuid,
  pos: Position,
  net: NetStatus
}

impl PlayerState {
  pub fn connect(&mut self) { self.net.connect() }
  pub fn connected(&self) -> bool { self.net.connected() }
  pub fn disconnect(&mut self) { self.net.disconnect() }

  pub fn new(source: network::Address) -> PlayerState {
    PlayerState {
      uuid: Uuid::new(UuidVersion::Sha1).unwrap(),
      pos: Position::origin(),
      net: NetStatus::new_connected(source)
    }
  }

  pub fn uuid(&self) -> Uuid {
    self.uuid.clone()
  }
}

#[derive(Clone)]
pub struct GameState {
  running: bool,
  players: HashMap<Uuid, PlayerState>,
  address_to_player: HashMap<network::Address, Uuid>
}

impl GameState {

  pub fn new() -> GameState {
    GameState {
      running: false,
      players: HashMap::new(),
      address_to_player: HashMap::new()
    }
  }

  pub fn add_player(&mut self, source: network::Address) -> PlayerState {
    let player = PlayerState::new(source);
    self.players.insert(player.uuid.clone(), player.clone());
    self.address_to_player.insert(source.clone(), player.uuid.clone());
    player
  }

  pub fn get_mut_player(&mut self, source: network::Address) -> Option<&mut PlayerState> {
    let uuid_opt = self.address_to_player.get(&source);
    if uuid_opt.is_some() {
      self.players.get_mut(uuid_opt.unwrap())
    } else {
      None
    }
  }

  pub fn get_player(&self, source: network::Address) -> Option<PlayerState> {
    self.address_to_player.get(&source)
      .and_then(|uuid| self.players.get(uuid).map(|x| x.clone()))
  }

  pub fn end(&mut self) {
    self.running = false;
  }

  pub fn start(&mut self) {
    self.running = true;
  }
}
