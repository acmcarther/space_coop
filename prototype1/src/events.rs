pub use self::events::{
  ClientEvent,
  ServerEvent,
  EntEvent
};

mod events {
  #[derive(Clone)]
  pub enum ClientEvent {
    KeepAlive,
    Connect,
    Disconnect,
    Chat { message: String },
    MoveSelf { x: f32, y: f32 },
    SetOwnColor { r: u8, g: u8, b: u8}
  }

  #[derive(Clone)]
  pub enum ServerEvent {
    KeepAlive,
    Connected { eid: u8 },
    NotConnected,
    Chatted { subject: String, message: String },
    EntEvent { eid: u8, event: EntEvent },
  }

  #[derive(Clone)]
  pub enum EntEvent {
    Spawned,
    Moved { x: f32, y: f32 },
    Recolored { r: u8, g: u8, b: u8 },
    Destroyed,
  }
}
