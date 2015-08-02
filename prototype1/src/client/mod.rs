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
  use std::io::stdin;
  use std::sync::mpsc::channel;

  use itertools::Itertools;
  use time::SteadyTime;

  pub fn start() {
    let mut state = ClientState::new();
    let client_params = params::query_client_params();
    let app_network = ClientNet::new(client_params.addr, client_params.server_addr);
    println!("Hello client!");
    app_network.send_event(ClientEvent::Connect);
    let (stdin_tx, stdin_rx) = channel();
    let mut last_sent = SteadyTime::now();

    thread::spawn (move || {
      let mut stdin = stdin();
      println!("type your messages");
      loop {
        let mut message = String::new();
        let _ = stdin.read_line(&mut message);
        let _ = stdin_tx.send(message);
      }
    });

    loop {
      thread::sleep_ms(20);

      let possible_command = stdin_rx.try_recv().ok();
      possible_command.map (|message| {
        if message.starts_with("say ") {
          app_network.send_event(ClientEvent::Chat{message: message[4..].to_string()});
        } else if message.starts_with("move") {
          app_network.send_event(ClientEvent::TryMove{x: 5.0, y: 5.0});
        }
      });

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
            ServerEvent::Chatted {subject, message} => println!("{}: {}", subject, message.trim()),
            ServerEvent::Moved { x, y } => {
              state.position = (x, y);
              println!("Moved to {:?}", (x, y))
            },
            _ => ()
          }
        })
    }
    //app_network.send_event(ClientEvent::Disconnect);
  }
}
