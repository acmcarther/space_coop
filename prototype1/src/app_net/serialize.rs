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
      ClientEvent::Connect => {
        vec![1]
      },
      ClientEvent::Disconnect => {
        vec![2]
      },
      ClientEvent::Chat => {
        vec![3]
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
      ServerEvent::Chatted => {
        vec![3]
      }
      ServerEvent::Moved => {
        vec![4]
      }
    }
  }
}
