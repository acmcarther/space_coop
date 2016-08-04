use specs;
use glutin;

use engine;

use common::protocol::ClientNetworkEvent;
use common::protocol::ServerNetworkEvent;
use common::protocol::SnapshotEvent;

use common::world::{DisabledAspect, PhysicalAspect, RenderAspect, SynchronizedAspect};
use engine::connection::ConnectionEvent;
use engine::control::player::MoveEvent;
use engine::control::camera::CameraMoveEvent;
use engine::control::menu::{MenuEvent, MenuState};
use engine::debug::DebugMessage;

// TODO(acmcarther): move somewhere more appropriate
#[derive(Debug, Clone)]
pub struct CameraPos(pub f32, pub f32, pub f32);
#[derive(Debug, Clone)]
pub struct OwnEntity(pub SynchronizedAspect);
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
    w.add_resource::<ExitFlag>(ExitFlag(false));
    w.add_resource::<CameraPos>(CameraPos(3.0, -10.0, 6.0));
    w.add_resource::<MenuState>(MenuState::new());
    w.add_resource::<DebugMessage>(DebugMessage("".to_owned()));
    w.add_resource::<Option<OwnEntity>>(None);
    w.add_resource::<Vec<ServerNetworkEvent>>(Vec::new());
    w.add_resource::<Vec<ConnectionEvent>>(Vec::new());
    w.add_resource::<Vec<ClientNetworkEvent>>(Vec::new());
    w.add_resource::<Vec<MoveEvent>>(Vec::new());
    w.add_resource::<Vec<CameraMoveEvent>>(Vec::new());
    w.add_resource::<Vec<SnapshotEvent>>(Vec::new());
    w.add_resource::<Vec<MenuEvent>>(Vec::new());
    w.add_resource::<Vec<glutin::Event>>(Vec::new());
    w.add_resource::<glutin::Window>(window);
    w.add_resource::<engine::connection::ConnectionStatus>(engine::connection::ConnectionStatus::new());

    World { world: w }
  }
}
