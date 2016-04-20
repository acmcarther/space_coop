use std::env;

use std::net::SocketAddr;

pub fn start(addr: SocketAddr) {
  println!("started a client on addr {}", addr.to_string());
}
