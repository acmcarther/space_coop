pub use self::state::{ClientState, ServerState};

mod state {
  use std::net::SocketAddr;
  use std::collections::HashMap;

  pub struct ClientState {
    pub position: (f32, f32)
  }

  pub struct ServerState {
    pub positions: HashMap<SocketAddr, (f32, f32)>
  }

  impl ServerState {
    pub fn new() -> ServerState {
      ServerState { positions: HashMap::new() }
    }
  }

  impl ClientState {
    pub fn new() -> ClientState {
      ClientState { position: (0.0, 0.0) }
    }
  }
}
