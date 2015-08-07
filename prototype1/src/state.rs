pub use self::state::{
  Primitive,
  ClientState,
  ServerState
};

mod state {
  use std::net::SocketAddr;
  use std::collections::HashMap;
  use time::SteadyTime;

  #[derive(Debug)]
  pub struct Primitive {
    pub color: (u8, u8, u8),
    pub pos: (f32, f32)
  }

  pub struct ClientState {
    pub entities: HashMap<u8, Primitive>,
    pub own_id: Option<u8>,
    pub zoom_level: u8
  }

  pub struct ServerState {
    pub connections: HashMap<SocketAddr, SteadyTime>,
    pub connection_to_entity: HashMap<SocketAddr, u8>,
    pub entities: HashMap<u8, Primitive>
  }

  impl ServerState {
    pub fn new() -> ServerState {
      ServerState {
        connections: HashMap::new(),
        entities: HashMap::new(),
        connection_to_entity: HashMap::new()
      }
    }
  }

  impl ClientState {
    pub fn new() -> ClientState {
      ClientState {
        entities: HashMap::new(),
        own_id: None,
        zoom_level: 1
      }
    }
  }
}
