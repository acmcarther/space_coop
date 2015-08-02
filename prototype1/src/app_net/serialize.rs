pub use self::serialize::{
  client_event,
  server_event,
};

mod serialize {
  use events::{
    ClientEvent,
    ServerEvent
  };

  use std::iter::repeat;

  pub fn client_event(event: ClientEvent) -> Vec<u8> {
    match event {
      ClientEvent::KeepAlive => {
        vec![0]
      },
      ClientEvent::Connect => {
        vec![1]
      },
      ClientEvent::Disconnect => {
        vec![2]
      },
      ClientEvent::Chat {message} => {
        vec![3]
          .into_iter()
          .chain(message.into_bytes().into_iter().take(200))
          .collect()
      },
      ClientEvent::TryMove => {
        vec![4]
      }
    }
  }

  pub fn server_event(event: ServerEvent) -> Vec<u8> {
    match event {
      ServerEvent::KeepAlive => {
        vec![0]
      },
      ServerEvent::Connected => {
        vec![1]
      },
      ServerEvent::NotConnected => {
        vec![2]
      },
      ServerEvent::Chatted {subject, message} => {
        vec![3]
          .into_iter()
          .chain(subject.into_bytes().into_iter().chain(repeat(0)).take(20))
          .chain(message.into_bytes().into_iter().take(200))
          .collect()

      }
      ServerEvent::Moved => {
        vec![4]
      }
    }
  }
}
