extern crate specs;
extern crate itertools;

extern crate common;
extern crate state;

use state::Delta;
use common::world::{DisabledAspect, PhysicalAspect, RenderAspect};

const FRAME_WAIT: u32 = 60;

pub struct DebugMessage(pub String);

/**
 * Useful for Debug
 */
pub struct System {
  frames_waited: u32,
}

impl System {
  pub fn new(world: &mut specs::World) -> System {
    world.add_resource::<DebugMessage>(DebugMessage("".to_owned()));

    // Render debug information in the first frame
    System { frames_waited: FRAME_WAIT }
  }

  pub fn name() -> &'static str {
    "debug"
  }
}

#[allow(unused_imports, unused_variables)]
impl specs::System<Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: Delta) {
    use specs::Join;
    use itertools::Itertools;

    let (entities, physical, disableds, renderables, mut debug_msg) = arg.fetch(|w| {
      (w.entities(),
       w.read::<PhysicalAspect>(),
       w.read::<DisabledAspect>(),
       w.read::<RenderAspect>(),
       w.write_resource::<DebugMessage>())
    });

    if self.frames_waited >= FRAME_WAIT {
      self.frames_waited = 0;

      let mut message = String::new();
      (&entities, &physical).iter().foreach(|(ent, phys)| {
        message.push_str(&format!("{:?}: {:?}\n", ent, phys));
      });

      *debug_msg = DebugMessage(message);
    } else {
      self.frames_waited += 1;
    }
  }
}
