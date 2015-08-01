pub use self::server::{start};

mod server {
  use params;
  use app_net::ServerNet;

  pub fn start() {
    let server_params = params::query_server_params();
    let app_network = ServerNet::new(server_params.addr);
    println!("Hello server!");
  }
}
