pub use self::client::{start};

mod client {
  use params;
  use app_net::ClientNet;

  pub fn start() {
    let client_params = params::query_client_params();
    let app_network = ClientNet::new(client_params.addr, client_params.server_addr);
    println!("Hello client!");
  }
}
