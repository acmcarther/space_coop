pub mod state;
pub mod engine;
pub mod event;
pub mod network;
pub mod world;

use std::collections::HashMap;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

use time;

use self::engine::Engine;
use self::network::Network;

// TODO: Tick rate
pub fn start(port: u32) {
  let engine = Engine::new();
  let network = Network::new(port);
  let running = true;
  let tick_rate = 66;
  let time_step = 1.0 / (tick_rate as f32); //ms

  /*
  let (tx, rx) =  mpsc::channel();

  thread::spawn(move || {
    
  })


  // Constantly monitor network
  // Every interval, collapse events into state
  // Notify clients
  */
  let next_time = time::now();
  while running {
    // sleep 2ms, check "tick now?" (defined as, is now after next_time?)
    // lots of work
    //
    // next_time = next_time + time_step
  }
}
