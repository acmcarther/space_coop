use specs;

use glutin;
use glutin::Event::KeyboardInput;
use std::convert::From;
use charsets::{CharAction, CharEvent, CharMotion};
use std::mem;
use common::Delta;
use pubsub::{PubSubStore, Publisher, SubscriberToken};

#[derive(Clone)]
pub struct ConsoleEvent(glutin::Event);

#[derive(Clone)]
pub struct CommandBuffer(pub String);

#[derive(Clone)]
pub struct CommandCursor(pub usize);

#[derive(Clone)]
pub struct ExecutedCommand(pub String);

impl ConsoleEvent {
  fn get_input(&self) -> &glutin::Event {
    let &ConsoleEvent(ref input) = self;
    input
  }
}

impl From<glutin::Event> for ConsoleEvent {
  fn from(event: glutin::Event) -> ConsoleEvent {
    ConsoleEvent(event)
  }
}

pub struct System {
  lshifted: bool,
  rshifted: bool,
  cursor: usize,
  capsed: bool,
  command_buffer: String,
  console_event_sub_token: SubscriberToken<ConsoleEvent>,
}

impl System {
  pub fn new(world: &mut specs::World) -> System {
    world.add_resource::<CommandBuffer>(CommandBuffer(String::new()));
    world.add_resource::<CommandCursor>(CommandCursor(0));

    System {
      lshifted: false,
      rshifted: false,
      capsed: false,
      cursor: 0,
      command_buffer: String::new(),
      console_event_sub_token: world.register_subscriber::<ConsoleEvent>(),
    }
  }

  pub fn name() -> &'static str {
    "console::input"
  }

  pub fn should_cap(&self) -> bool {
    let shift = self.lshifted || self.rshifted;
    (shift && !self.capsed) || (!shift && self.capsed)
  }

  pub fn handle_char_event(&mut self,
                           event: CharEvent,
                           executed_commands: &mut Publisher<ExecutedCommand>) {
    match event {
      CharEvent::Character(c) => {
        self.command_buffer.insert(self.cursor, c);
        self.cursor = self.cursor.saturating_add(1);
      },
      CharEvent::Motion(CharMotion::Left) => self.cursor = self.cursor.saturating_sub(1),
      CharEvent::Motion(CharMotion::Right) => self.cursor = self.cursor.saturating_add(1),
      CharEvent::Motion(CharMotion::Up) => {},
      CharEvent::Motion(CharMotion::Down) => {},
      CharEvent::Motion(CharMotion::Home) => self.cursor = 0,
      CharEvent::Motion(CharMotion::End) => self.cursor = self.command_buffer.len(),
      CharEvent::Action(CharAction::Backspace) => {
        if self.command_buffer.len() > 0 {
          self.command_buffer.remove(self.cursor.saturating_sub(1));
          self.cursor = self.cursor.saturating_sub(1);
        }
      },
      CharEvent::Action(CharAction::Delete) => {
        if self.command_buffer.len() > self.cursor {
          self.command_buffer.remove(self.cursor);
        }
      },
      CharEvent::Action(CharAction::Return) => {
        let mut buffer = String::new();
        mem::swap(&mut buffer, &mut self.command_buffer);
        self.cursor = 0;
        executed_commands.push(ExecutedCommand(buffer));
      },
      CharEvent::Invalid => {},
    };
  }
}

impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use charsets::ASCII_CHAR_DICT;
    use glutin::VirtualKeyCode::{Capital, LShift, RShift};
    use glutin::ElementState;
    use itertools::Itertools;

    let (mut console_events,
         mut console_command_buffer,
         mut console_cursor,
         mut executed_commands) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.console_event_sub_token).collected(),
       w.write_resource::<CommandBuffer>(),
       w.write_resource::<CommandCursor>(),
       w.fetch_publisher::<ExecutedCommand>())
    });

    console_events.drain(..).foreach(|event| {
      match event.get_input() {
        &KeyboardInput(ElementState::Pressed, _, Some(LShift)) => {
          self.lshifted = true;
        },
        &KeyboardInput(ElementState::Released, _, Some(LShift)) => {
          self.lshifted = false;
        },
        &KeyboardInput(ElementState::Pressed, _, Some(RShift)) => {
          self.rshifted = true;
        },
        &KeyboardInput(ElementState::Released, _, Some(RShift)) => {
          self.rshifted = false;
        },
        &KeyboardInput(ElementState::Pressed, _, Some(Capital)) => {
          self.capsed = !self.capsed;
        },
        &KeyboardInput(ElementState::Pressed, _, Some(ref key)) => {
          let event = ASCII_CHAR_DICT.get_mapping(self.should_cap(), key.clone());
          self.handle_char_event(event, &mut executed_commands);
        },
        &KeyboardInput(ElementState::Released, _, _) => {},
        _ => {}, // Something thats not a keyboard input
      }
    });

    // We're authoritative for the console command and the cursor -- any mutation
    // gets overriden
    *console_command_buffer = CommandBuffer(self.command_buffer.clone());
    *console_cursor = CommandCursor(self.cursor);
  }
}
