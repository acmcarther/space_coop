pub mod io;
pub mod debug;
pub mod control;
pub mod connection;
pub mod state;
pub mod graphics;

use gfx_device_gl;
use glutin;
use std::net::SocketAddr;
use std::sync::mpsc::{self, Receiver, Sender};
use specs;
use time;
use world::{ExitFlag, World};
use gfx_window_glutin;
use gfx;
use gfx::Device;

use renderer::opengl::primitive::{ColorFormat, DepthFormat};

const NETWORK_IO_PRIORITY: specs::Priority = 100;
const CONTROL_WINDOW_INPUT_PRIORITY: specs::Priority = 90;
const NETWORK_EVENT_DISTRIBUTION_PRIORITY: specs::Priority = 80;
const CONTROL_EVENT_DISTRIBUTION_PRIORITY: specs::Priority = 70;
const CONTROL_PLAYER_PRIORITY: specs::Priority = 65;
const CONTROL_CAMERA_PRIORITY: specs::Priority = 64;
const CONNECTION_PRIORITY: specs::Priority = 60;
const STATE_SNAPSHOT_PRIORITY: specs::Priority = 50;
const GRAPHICS_RENDERING_PRIORITY: specs::Priority = 10;
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
  encoder: Option<gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>>,
  encoder_recv: Receiver<gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>>,
  encoder_send: Sender<gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>>,
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

    let (my_encoder_send, their_encoder_recv) = mpsc::channel();
    let (their_encoder_send, my_encoder_recv) = mpsc::channel();
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
    planner.add_system(control::window_input::System::new(),
                       "control::window_input",
                       CONTROL_WINDOW_INPUT_PRIORITY);
    planner.add_system(control::event_distribution::System::new(),
                       "control::event_distribution",
                       CONTROL_EVENT_DISTRIBUTION_PRIORITY);
    planner.add_system(graphics::System::new(their_encoder_recv,
                                             their_encoder_send,
                                             factory,
                                             main_color,
                                             main_depth),
                       "graphics",
                       GRAPHICS_RENDERING_PRIORITY);
    planner.add_system(control::player::System::new(),
                       "control::player",
                       CONTROL_PLAYER_PRIORITY);
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
      encoder: Some(encoder),
      encoder_recv: my_encoder_recv,
      encoder_send: my_encoder_send,
      network_kill_signal: network_kill_sender,
      running: true,
    }
  }

  pub fn tick(&mut self, dt: &time::Duration) {
    // Let the renderer draw stuff with our encoder
    self.encoder_send.send(self.encoder.take().unwrap()).unwrap();

    // Spin all services
    self.planner.dispatch(Delta {
      dt: dt.clone(),
      now: time::now(),
    });

    // Grab window and encoder from ECS and finalize the render
    {
      let window = self.planner.mut_world().write_resource::<glutin::Window>();
      let mut encoder = self.encoder_recv.recv().unwrap();
      encoder.flush(&mut self.device);
      window.swap_buffers().unwrap();
      self.device.cleanup();

      // Put the encoder back into our pocket
      self.encoder = Some(encoder);
    }

    // Check for that exit signal
    if let ExitFlag(true) = *self.planner.mut_world().read_resource::<ExitFlag>() {
      self.running = false;
    }
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
