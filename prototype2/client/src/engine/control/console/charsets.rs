use glutin;
use std::collections::HashMap;

#[derive(Clone)]
pub enum CharMotion {
  Left,
  Right,
  Down,
  Up,
  Home,
  End,
}

#[derive(Clone)]
pub enum CharAction {
  Backspace,
  Delete,
  Return,
}

#[derive(Clone)]
pub enum CharEvent {
  Character(char),
  Motion(CharMotion),
  Action(CharAction),
  Invalid,
}

pub struct CharDict {
  conversion_mapping: HashMap<(bool, glutin::VirtualKeyCode), CharEvent>,
}

impl CharDict {
  pub fn new(map: HashMap<(bool, glutin::VirtualKeyCode), CharEvent>) -> CharDict {
    CharDict { conversion_mapping: map }
  }


  pub fn get_mapping(&self, shift: bool, code: glutin::VirtualKeyCode) -> CharEvent {
    self.conversion_mapping.get(&(shift, code)).cloned().unwrap_or(CharEvent::Invalid)
  }
}

lazy_static! {
  pub static ref ASCII_CHAR_DICT: CharDict = build_ascii_dict();
}

fn build_ascii_dict() -> CharDict {
  use glutin::VirtualKeyCode::*;

  let mut map = HashMap::new();

  map.insert((false, A), CharEvent::Character('a'));
  map.insert((true, A), CharEvent::Character('A'));
  map.insert((false, B), CharEvent::Character('b'));
  map.insert((true, B), CharEvent::Character('B'));
  map.insert((false, C), CharEvent::Character('c'));
  map.insert((true, C), CharEvent::Character('C'));
  map.insert((false, D), CharEvent::Character('d'));
  map.insert((true, D), CharEvent::Character('D'));
  map.insert((false, E), CharEvent::Character('e'));
  map.insert((true, E), CharEvent::Character('E'));
  map.insert((false, F), CharEvent::Character('f'));
  map.insert((true, F), CharEvent::Character('F'));
  map.insert((false, G), CharEvent::Character('g'));
  map.insert((true, G), CharEvent::Character('G'));
  map.insert((false, H), CharEvent::Character('h'));
  map.insert((true, H), CharEvent::Character('H'));
  map.insert((false, I), CharEvent::Character('i'));
  map.insert((true, I), CharEvent::Character('I'));
  map.insert((false, J), CharEvent::Character('j'));
  map.insert((true, J), CharEvent::Character('J'));
  map.insert((false, K), CharEvent::Character('k'));
  map.insert((true, K), CharEvent::Character('K'));
  map.insert((false, L), CharEvent::Character('l'));
  map.insert((true, L), CharEvent::Character('L'));
  map.insert((false, M), CharEvent::Character('m'));
  map.insert((true, M), CharEvent::Character('M'));
  map.insert((false, N), CharEvent::Character('n'));
  map.insert((true, N), CharEvent::Character('N'));
  map.insert((false, O), CharEvent::Character('o'));
  map.insert((true, O), CharEvent::Character('O'));
  map.insert((false, P), CharEvent::Character('p'));
  map.insert((true, P), CharEvent::Character('P'));
  map.insert((false, Q), CharEvent::Character('q'));
  map.insert((true, Q), CharEvent::Character('Q'));
  map.insert((false, R), CharEvent::Character('r'));
  map.insert((true, R), CharEvent::Character('R'));
  map.insert((false, S), CharEvent::Character('s'));
  map.insert((true, S), CharEvent::Character('S'));
  map.insert((false, T), CharEvent::Character('t'));
  map.insert((true, T), CharEvent::Character('T'));
  map.insert((false, U), CharEvent::Character('u'));
  map.insert((true, U), CharEvent::Character('U'));
  map.insert((false, V), CharEvent::Character('v'));
  map.insert((true, V), CharEvent::Character('V'));
  map.insert((false, W), CharEvent::Character('w'));
  map.insert((true, W), CharEvent::Character('W'));
  map.insert((false, X), CharEvent::Character('x'));
  map.insert((true, X), CharEvent::Character('X'));
  map.insert((false, Y), CharEvent::Character('y'));
  map.insert((true, Y), CharEvent::Character('Y'));
  map.insert((false, Z), CharEvent::Character('z'));
  map.insert((true, Z), CharEvent::Character('Z'));
  map.insert((false, Key1), CharEvent::Character('1'));
  map.insert((true, Key1), CharEvent::Character('!'));
  map.insert((false, Key2), CharEvent::Character('2'));
  map.insert((true, Key2), CharEvent::Character('@'));
  map.insert((false, Key3), CharEvent::Character('3'));
  map.insert((true, Key3), CharEvent::Character('#'));
  map.insert((false, Key4), CharEvent::Character('4'));
  map.insert((true, Key4), CharEvent::Character('$'));
  map.insert((false, Key5), CharEvent::Character('5'));
  map.insert((true, Key5), CharEvent::Character('%'));
  map.insert((false, Key6), CharEvent::Character('6'));
  map.insert((true, Key6), CharEvent::Character('^'));
  map.insert((false, Key7), CharEvent::Character('7'));
  map.insert((true, Key7), CharEvent::Character('&'));
  map.insert((false, Key8), CharEvent::Character('8'));
  map.insert((true, Key8), CharEvent::Character('*'));
  map.insert((false, Key9), CharEvent::Character('9'));
  map.insert((true, Key9), CharEvent::Character('('));
  map.insert((false, Key0), CharEvent::Character('0'));
  map.insert((true, Key0), CharEvent::Character(')'));
  map.insert((false, Space), CharEvent::Character(' '));
  map.insert((true, Space), CharEvent::Character(' '));
  map.insert((false, Equals), CharEvent::Character('='));
  map.insert((true, Equals), CharEvent::Character('+'));
  map.insert((false, Apostrophe), CharEvent::Character('\''));
  map.insert((true, Apostrophe), CharEvent::Character('"'));
  map.insert((false, Backslash), CharEvent::Character('\\'));
  map.insert((true, Backslash), CharEvent::Character('|'));
  map.insert((false, Semicolon), CharEvent::Character(';'));
  map.insert((true, Semicolon), CharEvent::Character(':'));
  map.insert((false, Slash), CharEvent::Character('/'));
  map.insert((true, Slash), CharEvent::Character('?'));
  map.insert((false, LBracket), CharEvent::Character('['));
  map.insert((true, LBracket), CharEvent::Character('{'));
  map.insert((false, RBracket), CharEvent::Character(']'));
  map.insert((true, RBracket), CharEvent::Character('}'));
  map.insert((false, Subtract), CharEvent::Character('-'));
  map.insert((true, Subtract), CharEvent::Character('_'));
  map.insert((false, Grave), CharEvent::Character('`'));
  map.insert((true, Grave), CharEvent::Character('~'));
  map.insert((false, Comma), CharEvent::Character(','));
  map.insert((true, Comma), CharEvent::Character('<'));
  map.insert((false, Period), CharEvent::Character('.'));
  map.insert((true, Period), CharEvent::Character('>'));
  map.insert((false, Tab), CharEvent::Character('\t'));
  map.insert((true, Tab), CharEvent::Character('\t'));
  map.insert((false, Up), CharEvent::Motion(CharMotion::Up));
  map.insert((true, Up), CharEvent::Motion(CharMotion::Up));
  map.insert((false, Down), CharEvent::Motion(CharMotion::Down));
  map.insert((true, Down), CharEvent::Motion(CharMotion::Down));
  map.insert((false, Left), CharEvent::Motion(CharMotion::Left));
  map.insert((true, Left), CharEvent::Motion(CharMotion::Left));
  map.insert((false, Right), CharEvent::Motion(CharMotion::Right));
  map.insert((true, Right), CharEvent::Motion(CharMotion::Right));
  map.insert((false, Home), CharEvent::Motion(CharMotion::Home));
  map.insert((true, Home), CharEvent::Motion(CharMotion::Home));
  map.insert((false, End), CharEvent::Motion(CharMotion::End));
  map.insert((true, End), CharEvent::Motion(CharMotion::End));
  map.insert((false, Delete), CharEvent::Action(CharAction::Delete));
  map.insert((true, Delete), CharEvent::Action(CharAction::Delete));
  map.insert((false, Back), CharEvent::Action(CharAction::Backspace));
  map.insert((true, Back), CharEvent::Action(CharAction::Backspace));
  map.insert((false, Return), CharEvent::Action(CharAction::Return));
  map.insert((true, Return), CharEvent::Action(CharAction::Return));

  CharDict::new(map)
}
