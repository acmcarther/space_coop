pub use self::events::{ClientEvent, ServerEvent};

mod events {
  pub enum ClientEvent {
    KeepAlive,
    Chat,
    TryMove,
  }

  pub enum ServerEvent {
    KeepAlive,
    Chat,
    Moved
  }
}
