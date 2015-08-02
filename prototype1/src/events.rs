pub use self::events::{ClientEvent, ServerEvent};

mod events {
  #[derive(Clone)]
  pub enum ClientEvent {
    KeepAlive,
    Connect,
    Disconnect,
    Chat { message: String },
    TryMove { x: f32, y: f32 },
  }

  #[derive(Clone)]
  pub enum ServerEvent {
    KeepAlive,
    Connected,
    NotConnected,
    Chatted { subject: String, message: String },
    Moved { x: f32, y: f32 }
  }
}
