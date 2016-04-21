pub enum ServerNetworkEvent {
  Connected,
  Disconnected,
  KeepAlive,
  DomainEvent(ServerEvent),
}

pub enum ServerEvent {
  FullSnapshot {series: u32, idx: u32, count: u32, state_fragment: String}
}

pub enum ClientNetworkEvent {
  Connect,
  Disconnect,
  KeepAlive,
  DomainEvent(ClientEvent),
}

pub enum ClientEvent {
  SelfMove {x_d: f32, y_d: f32, z_d: f32}
}
