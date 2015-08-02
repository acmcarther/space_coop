pub use self::server::{start};

mod server {
  use std::thread;
  use std::collections::HashMap;
  use std::net::SocketAddr;

  use app_net::ServerNet;
  use params;
  use state::ServerState;

  use events::{
    ClientEvent,
    ServerEvent
  };

  use itertools::Itertools;
  use time::SteadyTime;

  pub fn start() {
    let server_params = params::query_server_params();
    let app_network = ServerNet::new(server_params.addr);
    println!("Hello server!");
    let mut server_state = ServerState::new();
    let mut connected_users = HashMap::new();
    let mut last_cull = SteadyTime::now();

    loop {
      thread::sleep_ms(20);
      let now = SteadyTime::now();

      if (now - last_cull).num_seconds() > 5 {
        last_cull = now;
        let dead_users =
          connected_users
            .iter()
            .filter(|&(_, last_contact): &(_, &SteadyTime)| (now - last_contact.clone()).num_seconds() > 5)
            .map(|(&user_addr, _): (&SocketAddr, _)| user_addr.clone())
            .collect::<Vec<SocketAddr>>();

        dead_users
          .iter()
          .foreach(|dead_user_addr| {
            connected_users.remove(&dead_user_addr);
          });
      }

      let events = app_network.get_events();
      events
        .into_iter()
        .foreach(|(source, event)| {
          if connected_users.contains_key(&source) {
            connected_users.insert(source.clone(), SteadyTime::now());

            match event {
              ClientEvent::KeepAlive => {
                app_network.send_event(source.clone(), ServerEvent::KeepAlive);
              },
              ClientEvent::Connect => {app_network.send_event(source, ServerEvent::Connected);},
              ClientEvent::Disconnect => {
                connected_users.remove(&source);
                app_network.send_event(source, ServerEvent::NotConnected);
              },
              ClientEvent::Chat { message }  => {
                let event = ServerEvent::Chatted {subject: source.to_string(), message: message };
                connected_users.keys().foreach(|user_addr| {
                  app_network.send_event(user_addr.clone(), event.clone());
                })
              },
              ClientEvent::TryMove => println!("{:?} tried to move", source),
            }
          } else {
            match event {
              ClientEvent::Connect => {
                connected_users.insert(source.clone(), SteadyTime::now());
                app_network.send_event(source, ServerEvent::Connected);
              },
              _ => {app_network.send_event(source, ServerEvent::NotConnected);}
            }
          }
        })
    }
  }
}
