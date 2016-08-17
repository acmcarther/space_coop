use std::net::SocketAddr;
use std::sync::mpsc::{self, Sender};

use gfx_device_gl;
use glutin;
use specs;
use time;
use gfx_window_glutin;
use gfx;

use network;
use console;
use pause;
use debug;
use camera;
use renderer;
use synchronization;
use player;
use mutator;
use mouse_lock;

use renderer::opengl::OpenGlRenderer;
use renderer::opengl::primitive3d::{ColorFormat, DepthFormat};
use state::Delta;
use world::World;
use pubsub::PubSubStore;
use state::ExitFlag;

// Window input implicit priority: infinity
pub const NETWORK_IO_PRIORITY: specs::Priority = 100;
pub const NETWORK_EVENT_DISTRIBUTION_PRIORITY: specs::Priority = 90;
pub const PAUSE_PRIORITY: specs::Priority = 85;
pub const PLAYER_PREPROCESSOR_PRIORITY: specs::Priority = 77;
pub const MOUSE_LOCK_PRIORITY: specs::Priority = 77;
pub const CONSOLE_PREPROCESSOR_PRIORITY: specs::Priority = 77;
pub const CAMERA_PREPROCESSOR_PRIORITY: specs::Priority = 76;
pub const PLAYER_MOVE_PRIORITY: specs::Priority = 65;
pub const CAMERA_MOVE_PRIORITY: specs::Priority = 65;
pub const CONSOLE_INPUT_PRIORITY: specs::Priority = 65;
pub const CONSOLE_INVOKER_PRIORITY: specs::Priority = 64;
pub const MUTATOR_PRIORITY: specs::Priority = 63;
pub const NETWORK_CONNECTION_PRIORITY: specs::Priority = 60;
pub const STATE_SNAPSHOT_PRIORITY: specs::Priority = 50;
pub const NETWORK_KEEP_ALIVE_PRIORITY: specs::Priority = 5;
pub const DEBUG_PRIORITY: specs::Priority = 1;
// Renderer implicit priority: 0

pub struct Engine {
  pub planner: specs::Planner<Delta>,
  device: gfx_device_gl::Device,
  encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
  renderer: renderer::RenderingSystem,
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
    let mut world = World::new(window).world;

    // Instantiate systems with their pubsub and other resources
    let network_adapter_system =
      network::AdapterSystem::new(port, server_addr, network_kill_receiver, &mut world);
    let network_event_distribution_system = network::EventDistributionSystem::new(&mut world);
    let network_connection_system = network::ConnectionSystem::new(&mut world);
    let pause_system = pause::System::new(&mut world);
    let player_preprocessor_system = player::PreprocessorSystem::new(&mut world);
    let mouse_lock_system = mouse_lock::System::new(&mut world);
    let camera_preprocessor_system = camera::PreprocessorSystem::new(&mut world);
    let console_preprocessor_system = console::PreprocessorSystem::new(&mut world);
    let player_move_system = player::MoveSystem::new(&mut world);
    let camera_move_system = camera::MovementSystem::new(&mut world);
    let console_input_system = console::InputSystem::new(&mut world);
    let console_invoker_system = console::InvokeSystem::new(&mut world);
    let mutator_system = mutator::System::new(&mut world);
    let synchronization_system = synchronization::System::new(&mut world);
    let network_keep_alive_system = network::KeepAliveSystem::new(&mut world);
    let debug_system = debug::System::new(&mut world);

    // Insert systems into planner
    let mut planner = specs::Planner::new(world, 2 /* Threads, arbitrary */);
    planner.add_system(network_adapter_system,
                       network::AdapterSystem::name(),
                       NETWORK_IO_PRIORITY);
    planner.add_system(network_event_distribution_system,
                       network::EventDistributionSystem::name(),
                       NETWORK_EVENT_DISTRIBUTION_PRIORITY);
    planner.add_system(network_connection_system,
                       network::ConnectionSystem::name(),
                       NETWORK_CONNECTION_PRIORITY);
    planner.add_system(pause_system, pause::System::name(), PAUSE_PRIORITY);
    planner.add_system(player_preprocessor_system,
                       player::PreprocessorSystem::name(),
                       PLAYER_PREPROCESSOR_PRIORITY);
    planner.add_system(mouse_lock_system,
                       mouse_lock::System::name(),
                       MOUSE_LOCK_PRIORITY);
    planner.add_system(camera_preprocessor_system,
                       camera::PreprocessorSystem::name(),
                       CAMERA_PREPROCESSOR_PRIORITY);
    planner.add_system(console_preprocessor_system,
                       console::PreprocessorSystem::name(),
                       CONSOLE_PREPROCESSOR_PRIORITY);
    planner.add_system(player_move_system,
                       player::MoveSystem::name(),
                       PLAYER_MOVE_PRIORITY);
    planner.add_system(camera_move_system,
                       camera::MovementSystem::name(),
                       CAMERA_MOVE_PRIORITY);
    planner.add_system(console_input_system,
                       console::InputSystem::name(),
                       CONSOLE_INPUT_PRIORITY);
    planner.add_system(console_invoker_system,
                       console::InvokeSystem::name(),
                       CONSOLE_INVOKER_PRIORITY);
    planner.add_system(mutator_system, mutator::System::name(), MUTATOR_PRIORITY);
    planner.add_system(synchronization_system,
                       synchronization::System::name(),
                       STATE_SNAPSHOT_PRIORITY);
    planner.add_system(network_keep_alive_system,
                       network::KeepAliveSystem::name(),
                       NETWORK_KEEP_ALIVE_PRIORITY);
    planner.add_system(debug_system, "debug", DEBUG_PRIORITY);

    Engine {
      planner: planner,
      device: device,
      renderer: renderer::RenderingSystem::new(OpenGlRenderer::new(factory,
                                                                   main_color,
                                                                   main_depth)),
      encoder: encoder,
      network_kill_signal: network_kill_sender,
      running: true,
    }
  }

  pub fn tick(&mut self, dt: &time::Duration) {
    use itertools::Itertools;

    // Do the window poll in main thread (because of OSX issue)
    {
      let w = self.planner.mut_world();
      let (window, mut glutin_events) = (w.write_resource::<glutin::Window>(),
                                         w.fetch_publisher::<glutin::Event>());
      window.poll_events().into_iter().foreach(|e| glutin_events.push(e));
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
    let mut world = self.planner.mut_world();
    self.renderer.run(&mut world, &mut self.encoder, &mut self.device);
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
