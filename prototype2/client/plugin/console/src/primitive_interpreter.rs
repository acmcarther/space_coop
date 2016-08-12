use common::model::ModelType;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Command {
  Help,
  Exit,
  ListEntities,
  ShowEntity(String),
  CreateEntity,
  DeleteEntity(String),
  SetEntityPos(String, (f32, f32, f32)),
  SetEntityModel(String, ModelType),
}

impl Command {
  pub fn print_all() -> String {
    "help, exit, list_entities, create, show $id, delete $id, set_pos $id $x $y $z, set_model \
     (cube|sphere)"
      .to_owned()
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

    let split_message = message.split_whitespace().collect::<Vec<&str>>();

    match split_message.as_slice() {
      &["exit"] => InterpreterResult::Valid(Command::Exit),
      &["help"] => InterpreterResult::Valid(Command::Help),
      &["list_entities"] => InterpreterResult::Valid(Command::ListEntities),
      &["show", item] => InterpreterResult::Valid(Command::ShowEntity(item.to_owned())),
      &["create"] => InterpreterResult::Valid(Command::CreateEntity),
      &["delete", item] => InterpreterResult::Valid(Command::DeleteEntity(item.to_owned())),
      &["set_pos", item, x, y, z] => {
        match (f32::from_str(x), f32::from_str(y), f32::from_str(z)) {
          (Ok(x), Ok(y), Ok(z)) => {
            InterpreterResult::Valid(Command::SetEntityPos(item.to_owned(), (x, y, z)))
          },
          _ => InterpreterResult::Invalid,
        }
      },
      &["set_model", item, "cube"] => {
        InterpreterResult::Valid(Command::SetEntityModel(item.to_owned(), ModelType::Cube))
      },
      &["set_model", item, "sphere"] => {
        InterpreterResult::Valid(Command::SetEntityModel(item.to_owned(), ModelType::Icosphere3))
      },
      _ => InterpreterResult::Invalid,
    }
  }
}
