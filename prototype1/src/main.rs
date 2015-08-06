#![feature(bitvec)]
#![feature(lookup_host)]
#![feature(ip_addr)]
extern crate time;
extern crate byteorder;
extern crate game_udp;
extern crate itertools;
extern crate cgmath;
extern crate glutin;
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;

mod app_net;
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
