pub use self::events::{ClientEvent, ServerEvent};

mod events {
  #[derive(Clone)]
  pub enum ClientEvent {
    KeepAlive,
    Connect,
    Disconnect,
    Chat { message: String },
    TryMove { x: f32, y: f32 },
    SetColor { r: u8, g: u8, b: u8}
  }

  #[derive(Clone)]
  pub enum ServerEvent {
    KeepAlive,
    Connected,
    NotConnected,
    Chatted { subject: String, message: String },
    Moved { x: f32, y: f32 },
    ColorIs { r: u8, g: u8, b: u8}
  }
}
