extern crate time;
extern crate game_udp;

mod app_net_protocol;
mod simple_rendering;
mod state;

use app_net_protocol::do_network;
use simple_rendering::{
  ConsoleRenderer,
  Renderer
};
use state::{
  Pos,
  GameState
};

fn main() {
  //loop {
    //do_network();
    //do_control();
    let renderer = ConsoleRenderer {
      nothing: 5
    };
    renderer.render(&GameState { man_pos: Pos {x: 0.0, y: 0.0 } } );
  //}
}
