pub use self::client::{start};

mod client {
  use params;

  use app_net::ClientNet;
  use events::{
    ClientEvent,
    ServerEvent
  };
  use state::ClientState;

  use std::thread;

  use itertools::Itertools;
  use time::SteadyTime;

  pub fn start() {
    let state = ClientState::new();
    let client_params = params::query_client_params();
    let app_network = ClientNet::new(client_params.addr, client_params.server_addr);
    println!("Hello client!");
    app_network.send_event(ClientEvent::Connect);
    let mut last_sent = SteadyTime::now();
    loop {
      thread::sleep_ms(20);

      let now = SteadyTime::now();
      if (now - last_sent).num_seconds() > 1 {
        last_sent = now;
        app_network.send_event(ClientEvent::KeepAlive);
      }

      let events = app_network.get_events();
      events
        .into_iter()
        .foreach(|event| {
          match event {
            ServerEvent::Connected => println!("Connected"),
            ServerEvent::NotConnected => println!("Not Connected"),
            ServerEvent::Chatted => println!("Someone chattted"),
            ServerEvent::Moved => println!("I moved"),
            ServerEvent::KeepAlive => println!("I LIVE"),
            //_ => ()
          }
        })
    }
    app_network.send_event(ClientEvent::Disconnect);
  }
}
