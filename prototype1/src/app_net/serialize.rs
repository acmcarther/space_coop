pub use self::serialize::{
  client_event,
  server_event,
};

mod serialize {
  use events::{
    ClientEvent,
    ServerEvent
  };

  pub fn client_event(event: ClientEvent) -> Vec<u8> {
    match event {
      ClientEvent::KeepAlive => {
        vec![0]
      },
      ClientEvent::Chat => {
        vec![1]
      },
      ClientEvent::TryMove => {
        vec![2]
      }
    }
  }

  pub fn server_event(event: ServerEvent) -> Vec<u8> {
    match event {
      ServerEvent::KeepAlive => {
        vec![0]
      },
      ServerEvent::Chatted => {
        vec![1]
      },
      ServerEvent::Moved => {
        vec![2]
      }
    }
  }
}
