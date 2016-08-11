use specs;
use glutin;

use common::world::{DisabledAspect, OwnEntity, PhysicalAspect, RenderAspect, SynchronizedAspect};

#[derive(Debug, Clone)]
pub struct ExitFlag(pub bool);

// TODO(acmcarther): This is awkward... "world.world"
pub struct World {
  pub world: specs::World,
}

impl World {
  pub fn new(window: glutin::Window) -> World {
    let mut w = specs::World::new();

    w.register::<RenderAspect>();
    w.register::<PhysicalAspect>();
    w.register::<DisabledAspect>();
    w.register::<SynchronizedAspect>();

    // "Common" resources
    w.add_resource::<ExitFlag>(ExitFlag(false));
    w.add_resource::<glutin::Window>(window);
    w.add_resource::<Option<OwnEntity>>(None);

    World { world: w }
  }
}
