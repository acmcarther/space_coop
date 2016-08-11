#[derive(Debug, Clone)]
pub enum Command {
  Help,
  Exit,
}

impl Command {
  pub fn print_all() -> String {
    "help, exit".to_owned()
  }
}

#[derive(Debug, Clone)]
pub enum InterpreterResult {
  Valid(Command),
  Invalid,
}

pub struct Interpreter;

impl Interpreter {
  pub fn new() -> Interpreter {
    Interpreter
  }

  pub fn interpret(&mut self, message: String) -> InterpreterResult {
    use std::ops::Deref;

    match message.deref() {
      "exit" => InterpreterResult::Valid(Command::Exit),
      "help" => InterpreterResult::Valid(Command::Help),
      _ => InterpreterResult::Invalid,
    }
  }
}
