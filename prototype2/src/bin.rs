extern crate prototype2;

use std::env;
use std::str::FromStr;
use std::net::ToSocketAddrs;

static EXAMPLE_SERVER_COMMAND: &'static str = "space_coop server 8888";
static EXAMPLE_CLIENT_COMMAND: &'static str = "space_coop client 9999 192.168.0.1:8888";

fn main() {
  let app_type = env::args().nth(1);

  match app_type.as_ref().map(|v| v.as_ref()) {
    Some("server") => {
      let port_opt = env::args().nth(2).and_then(|v| u16::from_str(&v).ok());
      match port_opt {
        Some(port) => prototype2::server::start(port),
        None => {
          println!("ERROR: Specify a port, as in \"{}\"",
                   EXAMPLE_SERVER_COMMAND)
        },
      }
    },
    Some("client") => {
      let port_opt = env::args().nth(2).and_then(|v| u16::from_str(&v).ok());
      let socket_addr_opt = env::args()
        .nth(3)
        .and_then(|v| v.to_socket_addrs().ok())
        .and_then(|mut socket_addr_iter| socket_addr_iter.next());
      match (port_opt, socket_addr_opt) {
        (Some(port), Some(addr)) => prototype2::client::start(port, addr),
        _ => {
          println!("ERROR: Specify a local port and a server ip and port, as in \"{}\"",
                   EXAMPLE_CLIENT_COMMAND)
        },
      }
    },
    _ => {
      println!("ERROR: Specify an application type from [server, client], as in, \"{}\", or \"{}\"",
               EXAMPLE_SERVER_COMMAND,
               EXAMPLE_CLIENT_COMMAND)
    },
  }
}
