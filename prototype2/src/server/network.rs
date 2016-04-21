use std::net::SocketAddr;

pub type Address = SocketAddr;

pub struct Network {
  port: u32
}

impl Network {
  pub fn new(port: u32) -> Network {
    Network { port: port }
  }
}
