pub mod systems;

use automatic_system_installer::{AutoInstaller, Dag};
use gfx;

use gfx_device_gl;
use gfx_window_glutin;
use glutin;
use itertools::Itertools;
use network;
use pubsub::PubSubStore;
use renderer;
use renderer::opengl::OpenGlRenderer;
use renderer::opengl::primitive3d::{ColorFormat, DepthFormat};
use specs;
use state::Delta;
use state::ExitFlag;
use std::any::TypeId;
use std::net::SocketAddr;
use std::sync::mpsc::{self, Sender};
use time;
use world::World;

pub struct Engine {
  pub planner: specs::Planner<Delta>,
  device: gfx_device_gl::Device,
  encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
  renderer: renderer::RenderingSystem,
  network_kill_signal: Sender<()>,
  running: bool,
}

impl Engine {
  pub fn dependency_dag() -> Dag<TypeId> {
    let mut auto_installer = AutoInstaller::new();
    systems::install_auto_systems(&mut auto_installer);
    auto_installer.take_dag()
  }

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

    // Specially initialize the network adapter
    let network_adapter_system =
      network::AdapterSystem::new(port, server_addr, network_kill_receiver, &mut world);

    // Automatic system installation
    // TODO: chain these off each other when non-lexical borrows land
    // https://github.com/rust-lang/rust/issues/21906/
    let mut installer = AutoInstaller::with_world(world);
    installer.auto_install_instance(network_adapter_system);
    systems::install_auto_systems(&mut installer);
    let planner = installer.apply(5 /* Threads, arbitrary */);

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
    self.poll_window();
    self.dispatch_once(dt.clone());
    self.render();

    // Check for that exit signal
    if let ExitFlag(true) = *self.planner.mut_world().read_resource::<ExitFlag>() {
      self.running = false;
    }
  }

  pub fn poll_window(&mut self) {
    let w = self.planner.mut_world();
    let (window, mut glutin_events) = (w.write_resource::<glutin::Window>(),
                                       w.fetch_publisher::<glutin::Event>());
    window.poll_events().into_iter().foreach(|e| glutin_events.push(e));
  }

  pub fn dispatch_once(&mut self, dt: time::Duration) {
    // Spin all services
    self.planner.dispatch(Delta {
      dt: dt.clone(),
      now: time::now(),
    });

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
