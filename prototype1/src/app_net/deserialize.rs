pub use self::deserialize::{
  client_event,
  server_event,
};

mod deserialize {
  use events::{
    ClientEvent,
    ServerEvent
  };
  use byteorder::{ByteOrder, BigEndian};

  pub fn client_event(data: Vec<u8>) -> Option<ClientEvent> {
    data
      .get(0)
      .map(|marker| marker.clone())
      .and_then(|value| {
        match value {
          0 => Some(ClientEvent::KeepAlive),
          1 => Some(ClientEvent::Connect),
          2 => Some(ClientEvent::Disconnect),
          3 => {
            let full_msg = String::from_utf8(data).unwrap();
            let trimmed_msg = full_msg.trim_matches('\0').to_string();
            Some(ClientEvent::Chat{message: trimmed_msg})
          },
          4 => {
            let x_bytes = data.iter().skip(1).take(4).cloned().collect::<Vec<u8>>();
            let y_bytes = data.iter().skip(5).take(4).cloned().collect::<Vec<u8>>();
            if x_bytes.len() == 4 && y_bytes.len() == 4 {
              let x = BigEndian::read_f32(&x_bytes[..]);
              let y = BigEndian::read_f32(&y_bytes[..]);
              Some(ClientEvent::TryMove {x: x, y: y} )
            } else {
              None
            }
          },
          5 => Some(ClientEvent::SetColor { r: 0xFFu8, g: 0x00u8, b: 0x00u8 }),
          _ => None
        }
      })
  }

  // TODO: Make this function efficient
  pub fn server_event(data: Vec<u8>) -> Option<ServerEvent> {
    data
      .get(0)
      .map(|marker| marker.clone())
      .and_then(|value| {
        match value {
          0 => Some(ServerEvent::KeepAlive),
          1 => Some(ServerEvent::Connected),
          2 => Some(ServerEvent::NotConnected),
          3 => {
            // TODO: Use split_off when it stabilizes
            let subject_bytes = data.iter().skip(1).cloned().take(20).collect::<Vec<u8>>();
            let message_bytes = data.into_iter().skip(21).collect::<Vec<u8>>();

            let full_sub = String::from_utf8(subject_bytes).unwrap();
            let trimmed_sub = full_sub.trim_matches('\0').to_string();

            let full_msg = String::from_utf8(message_bytes).unwrap();
            let trimmed_msg = full_msg.trim_matches('\0').to_string();

            Some(ServerEvent::Chatted { subject: trimmed_sub, message: trimmed_msg})
          },
          4 => {
            let x_bytes = data.iter().skip(1).take(4).cloned().collect::<Vec<u8>>();
            let y_bytes = data.iter().skip(5).take(4).cloned().collect::<Vec<u8>>();
            if x_bytes.len() == 4 && y_bytes.len() == 4 {
              let x = BigEndian::read_f32(&x_bytes[..]);
              let y = BigEndian::read_f32(&y_bytes[..]);
              Some(ServerEvent::Moved {x: x, y: y} )
            } else {
              None
            }
          }
          5 => Some(ServerEvent::ColorIs { r: 0xFFu8, g: 0x00u8, b: 0x00u8 }),
          _ => None
        }
      })
  }
}
