pub use self::params::{
  query_server_params,
  query_client_params,
  ClientParams,
  ServerParams,
};

mod params {
  use str_ops::default_string;
  use net_helpers::{get_own_ip, get_ip};
  use std::io::stdin;
  use std::net::{SocketAddr};
  use std::str::FromStr;

  #[derive(Clone)]
  pub struct ClientParams {
    pub server_addr: SocketAddr,
    pub addr: SocketAddr,
  }

  #[derive(Clone)]
  pub struct ServerParams {
    pub addr: SocketAddr,
  }

  pub fn query_server_params() -> ServerParams {
    let mut stdin = stdin();
    let mut port_str = String::new();
    println!("Server Port (5555): ");
    let _ = stdin.read_line(&mut port_str);
    port_str = default_string(port_str.trim(), "5555");

    let local_ip = get_own_ip();
    let full_addr_string: String = local_ip.to_string() + ":" + &port_str;
    let addr = SocketAddr::from_str(&full_addr_string).unwrap();
    ServerParams{ addr: addr }
  }

  pub fn query_client_params() -> ClientParams {
    let local_ip = get_own_ip();
    let mut stdin = stdin();
    let mut client_port_str = String::new();
    let mut server_port_str = String::new();
    let mut server_addr_str = String::new();
    println!("Client Port (4444): ");
    let _ = stdin.read_line(&mut client_port_str);
    println!("Server Port (5555): ");
    let _ = stdin.read_line(&mut server_port_str);
    println!("Server Addr (localhost): ");
    let _ = stdin.read_line(&mut server_addr_str);

    client_port_str = default_string(client_port_str.trim(), "4444");
    server_port_str = default_string(server_port_str.trim(), "5555");
    server_addr_str = get_ip(&default_string(server_addr_str.trim(), "localhost")).to_string();

    let full_client_addr_string: String = local_ip.to_string() + ":" + &client_port_str;
    let full_server_addr_string: String = server_addr_str + ":" + &server_port_str;

    let addr: SocketAddr = SocketAddr::from_str(&full_client_addr_string).unwrap();
    let server_addr: SocketAddr = SocketAddr::from_str(&full_server_addr_string).unwrap();

    ClientParams {
      addr: addr,
      server_addr: server_addr
    }
  }
}
