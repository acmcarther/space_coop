pub use self::state::{ClientState, ServerState};

mod state {
  pub struct ClientState;
  pub struct ServerState;

  impl ServerState {
    pub fn new() -> ServerState {
      ServerState
    }
  }
}
