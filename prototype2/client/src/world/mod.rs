use common::ecs::aspects::{DisabledAspect, PhysicalAspect, RenderAspect, SynchronizedAspect};
use glutin;
use specs;
use state::{ExitFlag, OwnEntity};

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
