use specs;
use engine;
use common::world::{DisabledAspect, PhysicalAspect, RenderAspect};

/**
 * Useful for Debug
 */
pub struct System;

impl System {
  pub fn new() -> System {
    System
  }
}

#[allow(unused_imports, unused_variables)]
impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    use specs::Join;
    use itertools::Itertools;

    let (entities, physical, disableds, renderables) = arg.fetch(|w| {
      (w.entities(),
       w.read::<PhysicalAspect>(),
       w.read::<DisabledAspect>(),
       w.read::<RenderAspect>())
    });

    // (&entities, &physical).iter().foreach(|(ent, phys)| {
    // println!("{:?}: {:?}", ent, phys);
    // });
    //
  }
}
