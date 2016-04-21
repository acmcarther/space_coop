use server::network;
use uuid::Uuid;

pub enum GameEvent {
  World(WorldEvent),
  Ent(EntEvent),
  Command(CommandEvent),
}

pub enum WorldEvent {
  StartGame,
  EndGame,
}

pub enum EntEvent {
}

pub enum OOBEvent {
  
}

pub enum OutboundEvent {
  Directed {destination: Uuid, event: ClientEvent},
  DirectedOOB{ destination: network::Address, event: ClientEvent },
  Undirected(ClientEvent)
}

pub enum ClientEvent {
  StartGame,
  EndGame,
  EntMove{x: f32, y: f32, z: f32},
  PlayerJoined(Uuid),
  PlayerLeft(Uuid),
  Join(Uuid),
  Left,
  Invalid(InvalidCommand)
}

pub enum InvalidCommand {
  UnknownUuid(Uuid),
  AlreadyJoinedAs(Uuid),
  AlreadyConnected,
  NotJoined,
  Other(String)
}

pub struct CommandEvent {
  pub source: network::Address,
  pub command: Command,
}

pub enum Command {
  Join,
  Connect,
  Disconnect,
  MoveEnt {eid: Uuid, x: f32, y: f32, z: f32}
}
