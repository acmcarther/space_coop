pub use self::serialize::{
  client_event,
  server_event,
};

mod serialize {
  use events::{
    ClientEvent,
    ServerEvent
  };
  use byteorder::{ByteOrder, BigEndian};

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
      ClientEvent::TryMove {x, y} => {
        let mut x_bytes = [0; 4];
        let mut y_bytes = [0; 4];
        BigEndian::write_f32(&mut x_bytes, x);
        BigEndian::write_f32(&mut y_bytes, y);

        vec![4]
          .into_iter()
          .chain(x_bytes.iter().cloned())
          .chain(y_bytes.iter().cloned())
          .collect()
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
      ServerEvent::Moved {x, y} => {
        let mut x_bytes = [0; 4];
        let mut y_bytes = [0; 4];
        BigEndian::write_f32(&mut x_bytes, x);
        BigEndian::write_f32(&mut y_bytes, y);

        vec![4]
          .into_iter()
          .chain(x_bytes.iter().cloned())
          .chain(y_bytes.iter().cloned())
          .collect()
      }
    }
  }
}
