use common::aspects::{DisabledAspect, PhysicalAspect, RenderAspect, SynchronizedAspect};

use specs;
use aspects::{CollisionAspect, ControllerAspect, PlayerAspect};

pub struct ServerWorld {
  pub world: specs::World,
}

impl ServerWorld {
  pub fn new() -> ServerWorld {
    let mut w = specs::World::new();

    w.register::<RenderAspect>();
    w.register::<CollisionAspect>();
    w.register::<PhysicalAspect>();
    w.register::<DisabledAspect>();
    w.register::<PlayerAspect>();
    w.register::<ControllerAspect>();
    w.register::<SynchronizedAspect>();
    ServerWorld { world: w }
  }
}
