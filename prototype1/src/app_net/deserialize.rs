pub use self::deserialize::{
  client_event,
  server_event,
};

mod deserialize {
  use events::{
    ClientEvent,
    ServerEvent,
    EntEvent
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
              Some(ClientEvent::MoveSelf {x: x, y: y} )
            } else {
              None
            }
          },
          5 => {
            let has_colors = [data.get(1), data.get(2), data.get(3)].into_iter().cloned().filter(Option::is_some).count() == 3;
            if has_colors {
              Some(ClientEvent::SetOwnColor { r: data.get(1).unwrap().clone(), g: data.get(2).unwrap().clone(), b: data.get(3).unwrap().clone()})
            } else {
              None
            }
          },
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
          1 => data.get(1).map(|val| ServerEvent::Connected {eId: val.clone()} ),
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
            data.get(1).and_then (|eId| {
              data
                .get(2)
                .map(|ent_event_marker| ent_event_marker.clone())
                .and_then(|value| {
                  match value {
                    0 => {
                      Some(ServerEvent::EntEvent {eId: eId.clone(), event: EntEvent::Spawned})
                    },
                    1 => {
                      let x_bytes = data.iter().skip(3).take(4).cloned().collect::<Vec<u8>>();
                      let y_bytes = data.iter().skip(7).take(4).cloned().collect::<Vec<u8>>();
                      if x_bytes.len() == 4 && y_bytes.len() == 4 {
                        let x = BigEndian::read_f32(&x_bytes[..]);
                        let y = BigEndian::read_f32(&y_bytes[..]);
                        Some(ServerEvent::EntEvent {eId: eId.clone(), event: EntEvent::Moved {x: x, y: y} } )
                      } else {
                        None
                      }
                    }
                    2 => {
                      let has_colors = [data.get(3), data.get(4), data.get(5)].into_iter().cloned().filter(Option::is_some).count() == 3;
                      if has_colors {
                        Some(ServerEvent::EntEvent{ eId: eId.clone(), event: EntEvent::Recolored { r: data.get(3).unwrap().clone(), g: data.get(4).unwrap().clone(), b: data.get(5).unwrap().clone()}})
                      } else {
                        None
                      }
                    },
                    3 => Some(ServerEvent::EntEvent {eId: eId.clone(), event: EntEvent::Destroyed}),
                    _ => None
                  }
                })
            })
          }
          _ => None
        }
      })
  }
}
