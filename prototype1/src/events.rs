pub use self::events::{ClientEvent, ServerEvent};

mod events {
  pub enum ClientEvent {
    KeepAlive
  }

  pub enum ServerEvent {
    KeepAlive
  }
}
