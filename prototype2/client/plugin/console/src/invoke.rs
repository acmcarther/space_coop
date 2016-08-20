use specs;

use state::Delta;
use input::ExecutedCommand;
use std::collections::vec_deque::VecDeque;
use pubsub::{PubSubStore, SubscriberToken};
use primitive_interpreter::{Command, Interpreter, InterpreterResult};

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
pub struct System {
  executed_commands_sub_token: SubscriberToken<ExecutedCommand>,
  interpreter: Interpreter,
}
declare_dependencies!(System, [::input::System]);
standalone_installer_from_new!(System, Delta);

impl System {
  pub fn new(world: &mut specs::World) -> System {
    world.add_resource::<ConsoleLog>(ConsoleLog::new());
    System {
      executed_commands_sub_token: world.register_subscriber::<ExecutedCommand>(),
      interpreter: Interpreter::new(),
    }
  }

  pub fn name() -> &'static str {
    "console::invoke"
  }
}


impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use itertools::Itertools;

    let (mut executed_commands, mut translated_commands, mut console_log) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.executed_commands_sub_token).collected(),
       w.fetch_publisher::<Command>(),
       w.write_resource::<ConsoleLog>())
    });

    executed_commands.drain(..).foreach(|command| {
      let ExecutedCommand(command_message) = command;
      console_log.push(command_message.clone());
      match self.interpreter.interpret(command_message.clone()) {
        InterpreterResult::Valid(cmd) => translated_commands.push(cmd),
        InterpreterResult::Invalid => {
          console_log.push(format!("Unknown command: {}", &command_message));
        },
      }
    });
  }
}
