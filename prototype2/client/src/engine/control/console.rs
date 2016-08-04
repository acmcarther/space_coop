use specs;

use engine;

use glutin;
use glutin::Event::KeyboardInput;
use glutin::VirtualKeyCode;
use std::convert::From;
use std::mem;

pub struct ConsoleEvent(glutin::Event);

pub struct CommandBuffer(pub String);
pub struct CommandCursor(pub usize);
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
}

impl System {
  pub fn new() -> System {
    System {
      lshifted: false,
      rshifted: false,
      capsed: false,
      cursor: 0,
      command_buffer: String::new(),
    }
  }

  pub fn should_cap(&self) -> bool {
    let shift = self.lshifted || self.rshifted;
    (shift && !self.capsed) || (!shift && self.capsed)
  }

  pub fn handle_char_event(&mut self,
                           event: CharEvent,
                           executed_commands: &mut Vec<ExecutedCommand>) {
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

impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    use glutin::VirtualKeyCode::{Capital, LShift, RShift};
    use glutin::ElementState;
    use itertools::Itertools;

    let (mut console_events,
         mut console_command_buffer,
         mut console_cursor,
         mut executed_commands) = arg.fetch(|w| {
      (w.write_resource::<Vec<ConsoleEvent>>(),
       w.write_resource::<CommandBuffer>(),
       w.write_resource::<CommandCursor>(),
       w.write_resource::<Vec<ExecutedCommand>>())
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
          let event = input_to_char_event(key.clone(), self.should_cap());
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

pub enum CharMotion {
  Left,
  Right,
  Down,
  Up,
  Home,
  End,
}

pub enum CharAction {
  Backspace,
  Delete,
  Return,
}

pub enum CharEvent {
  Character(char),
  Motion(CharMotion),
  Action(CharAction),
  Invalid,
}

fn input_to_char_event(code: VirtualKeyCode, shift: bool) -> CharEvent {
  use glutin::VirtualKeyCode::*;

  match (shift, code) {
    (false, A) => CharEvent::Character('a'),
    (true, A) => CharEvent::Character('A'),
    (false, B) => CharEvent::Character('b'),
    (true, B) => CharEvent::Character('B'),
    (false, C) => CharEvent::Character('c'),
    (true, C) => CharEvent::Character('C'),
    (false, D) => CharEvent::Character('d'),
    (true, D) => CharEvent::Character('D'),
    (false, E) => CharEvent::Character('e'),
    (true, E) => CharEvent::Character('E'),
    (false, F) => CharEvent::Character('f'),
    (true, F) => CharEvent::Character('F'),
    (false, G) => CharEvent::Character('g'),
    (true, G) => CharEvent::Character('G'),
    (false, H) => CharEvent::Character('h'),
    (true, H) => CharEvent::Character('H'),
    (false, I) => CharEvent::Character('i'),
    (true, I) => CharEvent::Character('I'),
    (false, J) => CharEvent::Character('j'),
    (true, J) => CharEvent::Character('J'),
    (false, K) => CharEvent::Character('k'),
    (true, K) => CharEvent::Character('K'),
    (false, L) => CharEvent::Character('l'),
    (true, L) => CharEvent::Character('L'),
    (false, M) => CharEvent::Character('m'),
    (true, M) => CharEvent::Character('M'),
    (false, N) => CharEvent::Character('n'),
    (true, N) => CharEvent::Character('N'),
    (false, O) => CharEvent::Character('o'),
    (true, O) => CharEvent::Character('O'),
    (false, P) => CharEvent::Character('p'),
    (true, P) => CharEvent::Character('P'),
    (false, Q) => CharEvent::Character('q'),
    (true, Q) => CharEvent::Character('Q'),
    (false, R) => CharEvent::Character('r'),
    (true, R) => CharEvent::Character('R'),
    (false, S) => CharEvent::Character('s'),
    (true, S) => CharEvent::Character('S'),
    (false, T) => CharEvent::Character('t'),
    (true, T) => CharEvent::Character('T'),
    (false, U) => CharEvent::Character('u'),
    (true, U) => CharEvent::Character('U'),
    (false, V) => CharEvent::Character('v'),
    (true, V) => CharEvent::Character('V'),
    (false, W) => CharEvent::Character('w'),
    (true, W) => CharEvent::Character('W'),
    (false, X) => CharEvent::Character('x'),
    (true, X) => CharEvent::Character('X'),
    (false, Y) => CharEvent::Character('y'),
    (true, Y) => CharEvent::Character('Y'),
    (false, Z) => CharEvent::Character('z'),
    (true, Z) => CharEvent::Character('Z'),
    (false, Key1) => CharEvent::Character('1'),
    (true, Key1) => CharEvent::Character('!'),
    (false, Key2) => CharEvent::Character('2'),
    (true, Key2) => CharEvent::Character('@'),
    (false, Key3) => CharEvent::Character('3'),
    (true, Key3) => CharEvent::Character('#'),
    (false, Key4) => CharEvent::Character('4'),
    (true, Key4) => CharEvent::Character('$'),
    (false, Key5) => CharEvent::Character('5'),
    (true, Key5) => CharEvent::Character('%'),
    (false, Key6) => CharEvent::Character('6'),
    (true, Key6) => CharEvent::Character('^'),
    (false, Key7) => CharEvent::Character('7'),
    (true, Key7) => CharEvent::Character('&'),
    (false, Key8) => CharEvent::Character('8'),
    (true, Key8) => CharEvent::Character('*'),
    (false, Key9) => CharEvent::Character('9'),
    (true, Key9) => CharEvent::Character('('),
    (false, Key0) => CharEvent::Character('0'),
    (true, Key0) => CharEvent::Character(')'),
    (_, Space) => CharEvent::Character(' '),
    (false, Equals) => CharEvent::Character('='),
    (true, Equals) => CharEvent::Character('+'),
    (false, Apostrophe) => CharEvent::Character('\''),
    (true, Apostrophe) => CharEvent::Character('"'),
    (false, Backslash) => CharEvent::Character('\\'),
    (true, Backslash) => CharEvent::Character('|'),
    (false, Semicolon) => CharEvent::Character(';'),
    (true, Semicolon) => CharEvent::Character(':'),
    (false, Slash) => CharEvent::Character('/'),
    (true, Slash) => CharEvent::Character('?'),
    (false, LBracket) => CharEvent::Character('['),
    (true, LBracket) => CharEvent::Character('{'),
    (false, RBracket) => CharEvent::Character(']'),
    (true, RBracket) => CharEvent::Character('}'),
    (false, Subtract) => CharEvent::Character('-'),
    (true, Subtract) => CharEvent::Character('_'),
    (false, Grave) => CharEvent::Character('`'),
    (true, Grave) => CharEvent::Character('~'),
    (false, Comma) => CharEvent::Character(','),
    (true, Comma) => CharEvent::Character('<'),
    (false, Period) => CharEvent::Character('.'),
    (true, Period) => CharEvent::Character('>'),
    (_, Tab) => CharEvent::Character('\t'),
    (_, Up) => CharEvent::Motion(CharMotion::Up),
    (_, Down) => CharEvent::Motion(CharMotion::Down),
    (_, Left) => CharEvent::Motion(CharMotion::Left),
    (_, Right) => CharEvent::Motion(CharMotion::Right),
    (_, Home) => CharEvent::Motion(CharMotion::Home),
    (_, End) => CharEvent::Motion(CharMotion::End),
    (_, Delete) => CharEvent::Action(CharAction::Delete),
    (_, Back) => CharEvent::Action(CharAction::Backspace),
    (_, Return) => CharEvent::Action(CharAction::Return),
    _ => CharEvent::Invalid,
  }
}
