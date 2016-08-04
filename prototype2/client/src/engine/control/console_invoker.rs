use specs;
use glutin;

use engine;

use engine::control::console::ExecutedCommand;
use std::collections::vec_deque::VecDeque;

const COMMAND_HISTORY_CAPACITY: usize = 100;

pub struct CommandHistory {
  commands: VecDeque<String>,
}

impl CommandHistory {
  pub fn new() -> CommandHistory {
    CommandHistory { commands: VecDeque::with_capacity(COMMAND_HISTORY_CAPACITY) }
  }

  pub fn push(&mut self, command: String) {
    if self.commands.len() == COMMAND_HISTORY_CAPACITY {
      self.commands.pop_back();
    }

    self.commands.push_front(command);
  }

  pub fn list(&self, count: usize) -> Vec<&str> {
    self.commands.iter().map(|v| v.as_str()).take(count).collect()
  }
}

pub struct ConsoleLog {
  messages: VecDeque<String>,
}

impl ConsoleLog {
  pub fn new() -> ConsoleLog {
    ConsoleLog { messages: VecDeque::new() }
  }

  pub fn push(&mut self, message: String) {
    self.messages.push_front(message);
  }

  pub fn list(&self, count: usize) -> Vec<&str> {
    self.messages.iter().map(|v| v.as_str()).take(count).collect()
  }
}

/**
 * Handle commands emitted by console
 */
pub struct System;

impl System {
  pub fn new() -> System {
    System
  }
}


impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    use itertools::Itertools;

    let (mut executed_commands, mut command_history, mut console_log) = arg.fetch(|w| {
      (w.write_resource::<Vec<ExecutedCommand>>(),
       w.write_resource::<CommandHistory>(),
       w.write_resource::<ConsoleLog>())
    });

    executed_commands.drain(..).foreach(|command| {
      let ExecutedCommand(command_message) = command;
      console_log.push(command_message.clone());
      console_log.push(format!("Unknown command: {}", &command_message));
      command_history.push(command_message);
    });
  }
}
