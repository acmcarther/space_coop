pub use self::events::{ClientEvent, ServerEvent};

mod events {
  pub enum ClientEvent {
    KeepAlive,
    Connect,
    Disconnect,
    Chat,
    TryMove,
  }

  pub enum ServerEvent {
    KeepAlive,
    Connected,
    NotConnected,
    Chatted,
    Moved
  }
}
