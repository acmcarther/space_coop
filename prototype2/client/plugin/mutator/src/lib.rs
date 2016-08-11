extern crate specs;
extern crate itertools;
extern crate console;
extern crate pubsub;
extern crate state;

use state::Delta;
use console::{Command, ConsoleLog};
use pubsub::{PubSubStore, SubscriberToken};
use state::ExitFlag;

/**
 * Handle commands emitted by console
 */
pub struct System {
  commands_sub_token: SubscriberToken<Command>,
}

impl System {
  pub fn new(world: &mut specs::World) -> System {
    System { commands_sub_token: world.register_subscriber::<Command>() }
  }

  pub fn name() -> &'static str {
    "mutuator::System"
  }
}


impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use itertools::Itertools;

    let (mut commands, mut exit_flag, mut console_log) = arg.fetch(|w| {
      (w.fetch_subscriber(&self.commands_sub_token).collected(),
       w.write_resource::<ExitFlag>(),
       w.write_resource::<ConsoleLog>())
    });

    commands.drain(..).foreach(|e| {
      match e {
        Command::Exit => self.exit(&mut exit_flag, &mut console_log),
        Command::Help => self.help(&mut console_log),
      }
    });
  }
}

impl System {
  fn exit(&mut self, exit_flag: &mut ExitFlag, console_log: &mut ConsoleLog) {
    console_log.push("Exiting".to_owned());
    *exit_flag = ExitFlag(true);
  }

  fn help(&mut self, console_log: &mut ConsoleLog) {
    console_log.push(format!("Valid commands: {}", Command::print_all()));
  }
}
