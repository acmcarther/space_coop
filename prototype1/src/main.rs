#![feature(lookup_host)]
#![feature(ip_addr)]
extern crate time;
extern crate game_udp;
extern crate itertools;

mod app_net;
mod net_helpers;
mod client;
mod helpers;
mod server;
mod params;
mod str_ops;

mod events;
mod state;

use std::env;

fn main() {
  let app_type_str = env::args().nth(1).unwrap_or("client".to_string());

  if &app_type_str == "server" {
    server::start()
  } else {
    client::start()
  }
}
