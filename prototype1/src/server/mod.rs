pub use self::server::{start};

mod server {
  use std::thread;
  use std::collections::HashMap;
  use std::net::SocketAddr;
  use std::collections::BitVec;
  use std::collections::hash_map::{Entry};

  use app_net::ServerNet;
  use params;
  use state::{
    ServerState,
    Primitive,
  };

  use events::{
    ClientEvent,
    ServerEvent,
    EntEvent
  };

  use itertools::Itertools;
  use time::SteadyTime;

  pub fn start() {
    let server_params = params::query_server_params();
    let app_network = ServerNet::new(server_params.addr);
    println!("Hello server!");
    let mut server_state = ServerState::new();
    let mut last_cull = SteadyTime::now();

    loop {
      thread::sleep_ms(20);
      let now = SteadyTime::now();

      if (now - last_cull).num_seconds() > 5 {
        last_cull = now;
        let dead_users =
          server_state.connections
            .iter()
            .filter(|&(_, last_contact): &(_, &SteadyTime)| (now - last_contact.clone()).num_seconds() > 5)
            .map(|(&user_addr, _): (&SocketAddr, _)| user_addr.clone())
            .collect::<Vec<SocketAddr>>();

        dead_users
          .iter()
          .foreach(|dead_user_addr| {
            server_state.connections.remove(&dead_user_addr);
            let eId = server_state.connection_to_entity.remove(&dead_user_addr);
            eId.map(|eId| {
              server_state.connection_to_entity.remove(&dead_user_addr);
              server_state.entities.remove(&eId);
              server_state.connections.keys().foreach(|user_addr| {
                app_network.send_event(user_addr.clone(), ServerEvent::EntEvent{ eId: eId, event: EntEvent::Destroyed });
              });
            });
          });
      }

      let events = app_network.get_events();
      events
        .into_iter()
        .foreach(|(source, event)| {
          if server_state.connections.contains_key(&source) {
            server_state.connections.insert(source.clone(), SteadyTime::now());

            match event {
              ClientEvent::KeepAlive => {
                app_network.send_event(source.clone(), ServerEvent::KeepAlive);
              },
              ClientEvent::Connect => {
                server_state.connection_to_entity.get(&source).map(|eId| {
                  app_network.send_event(source, ServerEvent::Connected {eId: eId.clone()} );
                });
              },
              ClientEvent::Disconnect => {
                server_state.connections.remove(&source);
                server_state.connection_to_entity.remove(&source).map(|eId| {
                  server_state.entities.remove(&eId);
                  server_state.connections.keys().foreach(|user_addr| {
                    app_network.send_event(user_addr.clone(), ServerEvent::EntEvent{ eId: eId, event: EntEvent::Destroyed });
                  });
                });
                app_network.send_event(source, ServerEvent::NotConnected);
              },
              ClientEvent::Chat { message }  => {
                let event = ServerEvent::Chatted {subject: source.to_string(), message: message };
                server_state.connections.keys().foreach(|user_addr| {
                  app_network.send_event(user_addr.clone(), event.clone());
                })
              },
              ClientEvent::MoveSelf { x, y } => {
                println!("{:?} is moving to {:?}", source.clone(), (x, y));
                let connection_to_entity = &server_state.connection_to_entity;
                let connections = &server_state.connections;
                let entities = &mut server_state.entities;
                connection_to_entity.get(&source).map(|eId| {
                  match entities.entry(eId.clone()) {
                    Entry::Occupied(mut value) => {
                      let primitive = value.get_mut();
                      primitive.pos = (x, y);
                    },
                    _ => ()
                  }
                  connections.keys().foreach(|user_addr| {
                    println!("moving is getting sent");
                    app_network.send_event(user_addr.clone(), ServerEvent::EntEvent{ eId: eId.clone(), event: EntEvent::Moved { x: x, y: y }});
                  })
                });
              },
              ClientEvent::SetOwnColor { r, g, b } => {
                println!("{:?} is coloring to {:?}", source.clone(), (r, g, b));
                let connection_to_entity = &server_state.connection_to_entity;
                let connections = &server_state.connections;
                let entities = &mut server_state.entities;
                connection_to_entity.get(&source).map(|eId| {
                  match entities.entry(eId.clone()) {
                    Entry::Occupied(mut value) => {
                      let primitive = value.get_mut();
                      primitive.color = (r, g, b);
                    },
                    _ => ()
                  }
                  connections.keys().foreach(|user_addr| {
                    app_network.send_event(user_addr.clone(), ServerEvent::EntEvent{ eId: eId.clone(), event: EntEvent::Recolored { r: r, g: g, b: b }});
                  })
                });
              }
            }
          } else {
            match event {
              ClientEvent::Connect => {
                println!("connecting");
                let mut eId_in_use = BitVec::from_elem(256, false);
                let first_open_eId = server_state.connection_to_entity.values().cloned().foreach(|eId| {
                  eId_in_use.set(eId as usize, true);
                });
                println!("eId in use {:?}", eId_in_use);
                match eId_in_use.iter().enumerate().filter(|&(_, x)| !x).next() {
                  Some((eId, _)) => {
                    println!("got an eId {}", eId);
                    let eId = eId.clone() as u8;
                    server_state.connections.insert(source.clone(), SteadyTime::now());
                    server_state.connection_to_entity.insert(source.clone(), eId.clone());
                    server_state.entities.insert(eId.clone(), Primitive { color: (0, 0, 0), pos: (0.0, 0.0)});
                    app_network.send_event(source, ServerEvent::Connected {eId: eId} );
                  },
                  None => {
                    println!("no eid avail");
                    app_network.send_event(source, ServerEvent::NotConnected);
                  }
                };
              },
              _ => {app_network.send_event(source, ServerEvent::NotConnected);}
            }
          }
        })
    }
  }
}
