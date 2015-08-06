pub use self::state::{
  Primitive,
  ClientState,
  ServerState
};

mod state {
  use std::net::SocketAddr;
  use std::collections::HashMap;
  use time::SteadyTime;

  pub struct Primitive {
    pub color: (u8, u8, u8),
    pub pos: (f32, f32)
  }

  pub struct ClientState {
    pub position: (f32, f32),
    pub cube_color: (u8, u8, u8)
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
      ClientState { position: (0.0, 0.0), cube_color: (5, 5, 5) }
    }
  }
}
