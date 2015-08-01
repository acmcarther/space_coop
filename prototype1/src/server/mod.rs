pub use self::server::{start};

mod server {
  use std::thread;

  use app_net::ServerNet;
  use params;
  use state::ServerState;

  pub fn start() {
    let server_params = params::query_server_params();
    let app_network = ServerNet::new(server_params.addr);
    println!("Hello server!");
    let mut server_state = ServerState::new();

    loop {
      thread::sleep_ms(20);
      //app_network.integrate(&mut server_state);
    }
  }
}
