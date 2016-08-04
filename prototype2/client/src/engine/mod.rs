pub mod io;
pub mod debug;
pub mod control;
pub mod connection;
pub mod state;

use gfx_device_gl;
use glutin;
use std::net::SocketAddr;
use std::sync::mpsc::{self, Sender};
use specs;
use time;
use world::{ExitFlag, World};
use gfx_window_glutin;
use gfx;
use gfx::Device;

use renderer::opengl::OpenGlRenderer;
use renderer::opengl::primitive3d::{ColorFormat, DepthFormat};
use std::sync::RwLockReadGuard;
use std::ops::Not;
use world::{CameraPos, OwnEntity};
use common::world::{DisabledAspect, PhysicalAspect, RenderAspect, SynchronizedAspect};

const NETWORK_IO_PRIORITY: specs::Priority = 100;
const CONTROL_WINDOW_INPUT_PRIORITY: specs::Priority = 90;
const NETWORK_EVENT_DISTRIBUTION_PRIORITY: specs::Priority = 80;
const CONTROL_EVENT_DISTRIBUTION_PRIORITY: specs::Priority = 70;
const CONTROL_PLAYER_PRIORITY: specs::Priority = 65;
const CONTROL_MENU_PRIORITY: specs::Priority = 65;
const CONTROL_CAMERA_PRIORITY: specs::Priority = 65;
const CONNECTION_PRIORITY: specs::Priority = 60;
const STATE_SNAPSHOT_PRIORITY: specs::Priority = 50;
const NETWORK_HEALTH_CHECK_PRIORITY: specs::Priority = 5;
const DEBUG_PRIORITY: specs::Priority = 1;

#[derive(Debug, Clone)]
pub struct Delta {
  pub dt: time::Duration,
  pub now: time::Tm,
}

pub struct Engine {
  pub planner: specs::Planner<Delta>,
  device: gfx_device_gl::Device,
  encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
  factory: gfx_device_gl::Factory,
  renderer: OpenGlRenderer,
  network_kill_signal: Sender<()>,
  running: bool,
}

impl Engine {
  pub fn new(port: u16, server_addr: SocketAddr) -> Engine {
    // One time init gfx stuff
    let builder = glutin::WindowBuilder::new()
      .with_title("Space Coop".to_owned())
      .with_dimensions(1024, 768)
      .with_vsync();
    let (window, device, mut factory, main_color, main_depth) =
      gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
    let encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let (network_kill_sender, network_kill_receiver) = mpsc::channel();

    // ECS stuff
    let world = World::new(window);
    let mut planner = specs::Planner::new(world.world, 2 /* Threads, arbitrary */);

    planner.add_system(io::network_adapter::System::new(port, server_addr, network_kill_receiver),
                       "network::io",
                       NETWORK_IO_PRIORITY);
    planner.add_system(io::event_distribution::System::new(),
                       "network::event_distribution",
                       NETWORK_EVENT_DISTRIBUTION_PRIORITY);
    planner.add_system(connection::System::new(), "connection", CONNECTION_PRIORITY);
    // planner.add_system(control::window_input::System::new(),
    // "control::window_input",
    // CONTROL_WINDOW_INPUT_PRIORITY);
    //
    planner.add_system(control::event_distribution::System::new(),
                       "control::event_distribution",
                       CONTROL_EVENT_DISTRIBUTION_PRIORITY);
    planner.add_system(control::player::System::new(),
                       "control::player",
                       CONTROL_PLAYER_PRIORITY);
    planner.add_system(control::menu::System::new(),
                       "control::menu",
                       CONTROL_MENU_PRIORITY);
    planner.add_system(control::camera::System::new(),
                       "control::camera",
                       CONTROL_CAMERA_PRIORITY);
    planner.add_system(state::snapshot::System::new(),
                       "state::snapshot",
                       STATE_SNAPSHOT_PRIORITY);
    planner.add_system(io::health_check::System::new(),
                       "network::health_check",
                       NETWORK_HEALTH_CHECK_PRIORITY);
    planner.add_system(debug::System::new(), "debug", DEBUG_PRIORITY);

    Engine {
      planner: planner,
      device: device,
      renderer: OpenGlRenderer::new(factory.clone(), main_color, main_depth),
      factory: factory,
      encoder: encoder,
      network_kill_signal: network_kill_sender,
      running: true,
    }
  }

  pub fn tick(&mut self, dt: &time::Duration) {
    use specs::Join;
    use itertools::Itertools;

    // Do the window poll in main thread (because of OSX issue)
    {
      let mut w = self.planner.mut_world();
      let (window, mut glutin_events) = (w.write_resource::<glutin::Window>(),
                                         w.write_resource::<Vec<glutin::Event>>());
      glutin_events.extend(window.poll_events());
    }


    // Spin all services
    self.planner.dispatch(Delta {
      dt: dt.clone(),
      now: time::now(),
    });

    // This lives here because the renderer is not Send
    self.render();

    // Check for that exit signal
    if let ExitFlag(true) = *self.planner.mut_world().read_resource::<ExitFlag>() {
      self.running = false;
    }
  }

  pub fn render(&mut self) {
    let world = self.planner.mut_world();

    let (mut window,
         camera_pos,
         own_entity,
         debug_msg,
         menu_state,
         synchronized,
         entities,
         physical,
         disabled,
         render) = (world.write_resource::<glutin::Window>(),
                    world.read_resource::<CameraPos>(),
                    world.write_resource::<Option<OwnEntity>>(),
                    world.read_resource::<debug::DebugMessage>(),
                    world.read_resource::<control::menu::MenuState>(),
                    world.read::<SynchronizedAspect>(),
                    world.entities(),
                    world.read::<PhysicalAspect>(),
                    world.read::<DisabledAspect>(),
                    world.read::<RenderAspect>());

    let camera_target = SelfRetriever::new(own_entity.clone(), &entities, &synchronized, &physical)
      .find_own_pos();

    RenderWrapper::new(&mut self.renderer,
                       &mut self.encoder,
                       &mut window,
                       &camera_pos,
                       camera_target,
                       &debug_msg,
                       &menu_state,
                       &render,
                       &physical,
                       &disabled)
      .render_frame();

    self.encoder.flush(&mut self.device);
    window.swap_buffers().unwrap();
    self.device.cleanup();
  }

  pub fn running(&self) -> bool {
    self.running
  }

  pub fn finalize(&mut self, dt: &time::Duration) {
    // Tell the server we're leaving (to be polite)
    self.network_kill_signal.send(()).unwrap();

    // Spin all services
    self.tick(dt);
  }
}


// TODO(acmcarther): Move the below (all of it) somewhere else. This is here
// from the OpenGl service refactoring
type AspectStorageRead<'a, T> = specs::Storage<T,
                                               RwLockReadGuard<'a, specs::Allocator>,
                                               RwLockReadGuard<'a, specs::MaskedStorage<T>>>;

// TODO: Document
struct SelfRetriever<'a> {
  own_entity: Option<OwnEntity>,
  entities: &'a specs::Entities<'a>,
  synchronized: &'a AspectStorageRead<'a, SynchronizedAspect>,
  physical: &'a AspectStorageRead<'a, PhysicalAspect>,
}

impl<'a> SelfRetriever<'a> {
  pub fn new(own_entity: Option<OwnEntity>,
             entities: &'a specs::Entities<'a>,
             synchronized: &'a AspectStorageRead<'a, SynchronizedAspect>,
             physical: &'a AspectStorageRead<'a, PhysicalAspect>)
             -> SelfRetriever<'a> {

    SelfRetriever {
      own_entity: own_entity,
      entities: entities,
      synchronized: synchronized,
      physical: physical,
    }
  }

  pub fn find_own_pos(mut self) -> Option<(f32, f32, f32)> {
    use specs::Join;

    self.own_entity
      .take()
      .and_then(|ent| {
        (self.entities, self.synchronized)
          .iter()
          .filter(|&(_, synchro)| {
            let OwnEntity(ref own_ent) = ent;
            synchro == own_ent
          })
          .next()
      })
      .map(|(entity, _)| entity)
      .and_then(|true_ent| self.physical.get(true_ent))
      .map(|physical_aspect| physical_aspect.pos)
  }
}

// TODO(acmcarther): Document and rename to something more descriptive
// TODO(acmcarther): This seems like it defeats the purpose: it takes *way* too
// many params
struct RenderWrapper<'a> {
  renderer: &'a mut OpenGlRenderer,
  encoder: &'a mut gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
  window: &'a mut glutin::Window,
  camera_pos: &'a CameraPos,
  camera_target: Option<(f32, f32, f32)>,
  debug_msg: &'a debug::DebugMessage,
  menu_state: &'a control::menu::MenuState,
  render: &'a AspectStorageRead<'a, RenderAspect>,
  physical: &'a AspectStorageRead<'a, PhysicalAspect>,
  disabled: &'a AspectStorageRead<'a, DisabledAspect>,
}

impl<'a> RenderWrapper<'a> {
  pub fn new(renderer: &'a mut OpenGlRenderer,
             encoder: &'a mut gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
             window: &'a mut glutin::Window,
             camera_pos: &'a CameraPos,
             camera_target: Option<(f32, f32, f32)>,
             debug_msg: &'a debug::DebugMessage,
             menu_state: &'a control::menu::MenuState,
             render: &'a AspectStorageRead<'a, RenderAspect>,
             physical: &'a AspectStorageRead<'a, PhysicalAspect>,
             disabled: &'a AspectStorageRead<'a, DisabledAspect>)
             -> RenderWrapper<'a> {
    RenderWrapper {
      renderer: renderer,
      encoder: encoder,
      window: window,
      camera_pos: camera_pos,
      camera_target: camera_target,
      debug_msg: debug_msg,
      menu_state: menu_state,
      render: render,
      physical: physical,
      disabled: disabled,
    }
  }

  pub fn render_frame(mut self) {
    use specs::Join;
    use itertools::Itertools;

    let &CameraPos(x, y, z) = self.camera_pos;

    // TODO: clean up this api: we shouldnt rely on implict state setting here
    self.renderer.render_world(&mut self.encoder,
                               &mut self.window,
                               &(x, y, z),
                               self.camera_target);

    (self.physical, self.disabled.not(), self.render)
      .iter()
      .foreach(|(physical_aspect, _, render_aspect)| {
        self.renderer.render_model(&mut self.encoder,
                                   &mut self.window,
                                   physical_aspect,
                                   render_aspect,
                                   (x, y, z));
      });

    self.renderer.render_ui(&mut self.encoder,
                            &mut self.window,
                            &self.debug_msg,
                            &self.menu_state);
  }
}
