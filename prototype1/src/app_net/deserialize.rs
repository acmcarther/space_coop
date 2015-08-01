pub use self::deserialize::{
  client_event,
  server_event,
};

mod deserialize {
  use events::{
    ClientEvent,
    ServerEvent
  };

  pub fn client_event(data: Vec<u8>) -> Option<ClientEvent> {
    data
      .get(0)
      .and_then(|value| {
        match *value {
          0 => Some(ClientEvent::KeepAlive),
          1 => Some(ClientEvent::Connect),
          2 => Some(ClientEvent::Disconnect),
          3 => Some(ClientEvent::Chat),
          4 => Some(ClientEvent::TryMove),
          _ => None
        }
      })
  }

  pub fn server_event(data: Vec<u8>) -> Option<ServerEvent> {
    data
      .get(0)
      .and_then(|value| {
        match *value {
          0 => Some(ServerEvent::KeepAlive),
          1 => Some(ServerEvent::Connected),
          2 => Some(ServerEvent::NotConnected),
          3 => Some(ServerEvent::Chatted),
          4 => Some(ServerEvent::Moved),
          _ => None
        }
      })
  }
}
