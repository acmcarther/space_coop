pub use self::server::{start};

mod server {
  use std::thread;
  use std::collections::HashMap;

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

    loop {
      thread::sleep_ms(20);

      let events = app_network.get_events();
      events
        .into_iter()
        .foreach(|(source, event)| {
          if connected_users.contains_key(&source) {
            match event {
              ClientEvent::KeepAlive => {
                app_network.send_event(source.clone(), ServerEvent::KeepAlive);
                println!("{:?} LIVES!", source);
              },
              ClientEvent::Connect => {app_network.send_event(source, ServerEvent::Connected);},
              ClientEvent::Disconnect => {
                connected_users.remove(&source);
                app_network.send_event(source, ServerEvent::NotConnected);
              },
              ClientEvent::Chat => println!("{:?} chatted", source),
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
