use std::collections::HashMap;
use std::mem;

use server::state::GameState;
use server::world::ServerWorld;
use server::event::{
  GameEvent,
  EntEvent,
  WorldEvent,
  ClientEvent,
  CommandEvent,
  OutboundEvent,
  Command,
  InvalidCommand,
};

pub struct Engine {
  game: GameState,
  world: ServerWorld,
  events: Vec<GameEvent>
}

impl Engine {
  pub fn push_event(&mut self, event: GameEvent) { self.events.push(event) }

  pub fn new() -> Engine {
    Engine {
      world: ServerWorld::new(),
      game: GameState::new(),
      events: Vec::new()
    }
  }

  pub fn tick(&mut self) -> Vec<OutboundEvent> {
    let mut event_buf = Vec::new();
    // Yank all the events off the queue, replacing with a new queue
    mem::swap(&mut self.events, &mut event_buf);

    let mut outbound = Vec::new();

    for event in event_buf.drain(..) {
      match event {
        GameEvent::Ent(e) => outbound.append(&mut self.incorporate_ent_event(e)),
        GameEvent::World(e) => outbound.append(&mut self.incorporate_world_event(e)),
        GameEvent::Command(e) => outbound.append(&mut self.incorporate_command_event(e))
      }
    }

    outbound
  }

  fn incorporate_ent_event(&mut self, _: EntEvent) -> Vec<OutboundEvent> {
    Vec::new()
  }

  fn incorporate_world_event(&mut self, e: WorldEvent) -> Vec<OutboundEvent> {
    match e {
      WorldEvent::StartGame => {
        self.game.start();
        vec![OutboundEvent::Undirected(ClientEvent::StartGame)]
      },
      WorldEvent::EndGame => {
        self.game.end();
        vec![OutboundEvent::Undirected(ClientEvent::EndGame)]
      }
    }
  }

  fn incorporate_command_event(&mut self, e: CommandEvent) -> Vec<OutboundEvent> {
    match e.command {
      Command::Join => {
        let existing_player = self.game.get_player(e.source);
        if existing_player.is_some() {
          let player = existing_player.unwrap();
          vec![
            OutboundEvent::DirectedOOB{
              destination: e.source,
              event: ClientEvent::Invalid(InvalidCommand::AlreadyJoinedAs(player.uuid()))
            }
          ]
        } else {
          let new_player = self.game.add_player(e.source);
          vec![
            OutboundEvent::Directed{destination: new_player.uuid(), event: ClientEvent::Join(new_player.uuid())},
            OutboundEvent::Undirected(ClientEvent::PlayerJoined(new_player.uuid()))
          ]
        }
      },
      Command::Connect => {
        let existing_player = self.game.get_mut_player(e.source);
        match existing_player {
          Some(player) => {
            player.connect();
            vec![
              OutboundEvent::Directed{destination: player.uuid(), event: ClientEvent::Join(player.uuid())},
              OutboundEvent::Undirected(ClientEvent::PlayerJoined(player.uuid()))
            ]
          },
          None => {
            vec![
              OutboundEvent::DirectedOOB{
                destination: e.source,
                event: ClientEvent::Invalid(InvalidCommand::NotJoined)
              }
            ]
          }
        }
      },
      Command::Disconnect => {
        let existing_player = self.game.get_mut_player(e.source);
        match existing_player {
          Some(player) => {
            player.disconnect();
            vec![
              OutboundEvent::Directed{destination: player.uuid(), event: ClientEvent::Left},
              OutboundEvent::Undirected(ClientEvent::PlayerLeft(player.uuid()))
            ]
          }
          None => {
            vec![
              OutboundEvent::DirectedOOB{
                destination: e.source,
                event: ClientEvent::Invalid(InvalidCommand::NotJoined)
              }
            ]
          }
        }
      }
      _ => Vec::new()
    }
  }
}
