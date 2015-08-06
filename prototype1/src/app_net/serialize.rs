pub use self::serialize::{
  client_event,
  server_event,
};

mod serialize {
  use events::{
    ClientEvent,
    ServerEvent,
    EntEvent,
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
      ClientEvent::MoveSelf {x, y} => {
        let mut x_bytes = [0; 4];
        let mut y_bytes = [0; 4];
        BigEndian::write_f32(&mut x_bytes, x);
        BigEndian::write_f32(&mut y_bytes, y);

        vec![4]
          .into_iter()
          .chain(x_bytes.iter().cloned())
          .chain(y_bytes.iter().cloned())
          .collect()
      },
      ClientEvent::SetOwnColor {r, g, b} => {
        vec![5, r, g, b]
      },
    }
  }

  pub fn server_event(event: ServerEvent) -> Vec<u8> {
    match event {
      ServerEvent::KeepAlive => {
        vec![0]
      },
      ServerEvent::Connected { eId } => {
        vec![1, eId]
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
      ServerEvent::EntEvent { eId, event } => {
        let proto_msg = vec![4, eId];

        match event {
          EntEvent::Spawned => {
            proto_msg
              .into_iter()
              .chain([0].into_iter().cloned())
              .collect()
          },
          EntEvent::Moved {x, y} => {
            let mut x_bytes = [0; 4];
            let mut y_bytes = [0; 4];
            BigEndian::write_f32(&mut x_bytes, x);
            BigEndian::write_f32(&mut y_bytes, y);

            proto_msg
              .into_iter()
              .chain([1].into_iter().cloned())
              .chain(x_bytes.iter().cloned())
              .chain(y_bytes.iter().cloned())
              .collect()
          },
          EntEvent::Recolored {r, g, b} => {
            proto_msg
              .into_iter()
              .chain([2].into_iter().cloned())
              .chain([r, g, b].into_iter().cloned())
              .collect()
          },
          EntEvent::Destroyed => {
            proto_msg
              .into_iter()
              .chain([3].into_iter().cloned())
              .collect()
          }
        }
      }
    }
  }
}
