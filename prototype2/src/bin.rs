extern crate prototype2;

use std::env;
use std::str::FromStr;
use std::net::ToSocketAddrs;

fn main() {
  let app_type = env::args().nth(1);

  match app_type.as_ref().map(|v| v.as_ref()) {
    Some("server") => {
      let port_opt = env::args().nth(2).and_then(|v| u32::from_str(&v).ok());
      match port_opt {
        Some(port) => prototype2::server::start(port),
        None => println!("ERROR: Specify a port, as in \"space_coop server 8888\"")
      }
    },
    Some("client") => {
      let socket_addr_opt = env::args().nth(2)
        .and_then(|v| v.to_socket_addrs().ok())
        .and_then(|mut socket_addr_iter| socket_addr_iter.next());
      match socket_addr_opt {
        Some(addr) => prototype2::client::start(addr),
        None => println!("ERROR: Specify an ip and port, as in \"space_coop client 192.168.0.1:8888\"")
      }
    }
    _ => {
      println!("ERROR: Specify an application type from [server, client], as in, \"space_coop server\"")
    }
  }
}
