use specs;
use engine;
use std::ops::Deref;

pub struct System;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum MenuState {
  Open,
  Closed,
}

impl MenuState {
  pub fn new() -> MenuState {
    MenuState::Closed
  }

  pub fn is_open(&self) -> bool {
    self == &MenuState::Open
  }
}


pub enum MenuEvent {
  Open,
  Close,
}

impl System {
  pub fn new() -> System {
    System
  }
}

impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    use itertools::Itertools;

    let (mut menu_state, mut menu_events) =
      arg.fetch(|w| (w.write_resource::<MenuState>(), w.write_resource::<Vec<MenuEvent>>()));

    menu_events.drain(..).foreach(|e| {
      let result = match (e, menu_state.deref()) {
        (MenuEvent::Close, &MenuState::Open) => Some(MenuState::Closed),
        (MenuEvent::Open, &MenuState::Closed) => Some(MenuState::Open),
        _ => None,
      };
      result.map(|v| *menu_state = v);
    });
  }
}
