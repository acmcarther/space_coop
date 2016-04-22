use std::mem;

use itertools::Itertools;

use client::protocol::PartialSnapshot;

use common::world::ClientWorld;
use common::protocol::{ClientNetworkEvent, ServerNetworkEvent, ServerEvent};

use client::renderer::Renderer;
use client::controller::Controller;

pub struct Engine {
  renderer: Renderer,
  controller: Controller,
  events: Vec<ServerNetworkEvent>,
  partial_snapshot: Option<PartialSnapshot>,
  last_snapshot: Option<u16>,
  world: Option<ClientWorld>
}

impl Engine {
  pub fn push_event(&mut self, event: ServerNetworkEvent) { self.events.push(event) }

  pub fn new() -> Engine {
    Engine {
      renderer: Renderer::new(),
      controller: Controller::new(),
      events: Vec::new(),
      partial_snapshot: None,
      last_snapshot: None,
      world: None,
    }
  }

  pub fn tick(&mut self) -> Vec<ClientNetworkEvent> {
    let mut event_buf = Vec::new();
    // Yank all the events off the queue, replacing with a new queue
    mem::swap(&mut self.events, &mut event_buf);

    let mut outbound: Vec<ClientNetworkEvent> = Vec::new();

    event_buf.drain(..).foreach(|event| outbound.append(&mut self.handle(event)));

    outbound
  }

  fn handle(&mut self, event: ServerNetworkEvent) -> Vec<ClientNetworkEvent> {
    use common::protocol::ServerNetworkEvent::*;

    match event {
      DomainEvent(ServerEvent::FullSnapshot {series, idx, count, state_fragment}) => {
        let mut result = None;
        if self.other_snapshot_newer(series) {
          match self.partial_snapshot.as_mut() {
            None => {
              let partial_snapshot = PartialSnapshot::new(series, idx, count, state_fragment);
              if partial_snapshot.is_complete() {
                match partial_snapshot.collate() {
                  None => println!("weird, could not collate a snapshot. Thats extra weird because this is a one shot snapshot"),
                  Some(world) => {
                    self.world = Some(world);
                    result = Some(None);
                  }
                };
              } else {
                result = Some(Some(partial_snapshot));
              }
            },
            Some(existing_partial) => {
              if existing_partial.series == series {
                existing_partial.append(idx, state_fragment);
                if existing_partial.is_complete() {
                  match existing_partial.collate() {
                    None => println!("weird, could not collate a snapshot"),
                    Some(world) => {
                      self.world = Some(world);
                      result = Some(None);
                    }
                  };
                }
              } else if existing_partial.other_series_newer(series) {
                let partial_snapshot = PartialSnapshot::new(series, idx, count, state_fragment);
                if partial_snapshot.is_complete() {
                  match partial_snapshot.collate() {
                    None => println!("weird, could not collate a snapshot. Thats extra weird because this is a one shot snapshot"),
                    Some(world) => {
                      self.world = Some(world);
                      result = Some(None);
                    }
                  };
                } else {
                  result = Some(Some(partial_snapshot));
                }
              }
            }
          }
        }
        if result.is_some() { self.partial_snapshot = result.unwrap() }
        // TODO: Eventually, tell server we got their snapshot
        Vec::new()
      },
      KeepAlive => {
        //println!("Server acknowledged our keepalive");
        Vec::new()
      },
      _ => {
        println!("got something else");
        Vec::new()
      }
    }
  }

  pub fn other_snapshot_newer(&self, series: u16) -> bool {
    match self.last_snapshot {
      None => true,
      Some(u) => {
        let pos_diff = series.wrapping_sub(u);
        pos_diff != 0 && pos_diff < 32000
      }
    }
  }
}
