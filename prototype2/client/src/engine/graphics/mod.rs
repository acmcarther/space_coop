use specs;
use glutin;
use gfx_device_gl;
use gfx;

use std::sync::mpsc::{Receiver, Sender};
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Not;

use engine;
use renderer::opengl::OpenGlRenderer;
use renderer::opengl::primitive::{ColorFormat, DepthFormat};

use common::world::{DisabledAspect, PhysicalAspect, RenderAspect, SynchronizedAspect};
use world::{CameraPos, OwnEntity};
use gfx::handle::{DepthStencilView, RenderTargetView};

/**
 * Render to the window
 */
pub struct System {
  renderer: OpenGlRenderer,
  encoder_recv: Receiver<gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>>,
  encoder_send: Sender<gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>>,
}

impl System {
  pub fn new(encoder_recv: Receiver<gfx::Encoder<gfx_device_gl::Resources,
                                                 gfx_device_gl::CommandBuffer>>,
             encoder_send: Sender<gfx::Encoder<gfx_device_gl::Resources,
                                               gfx_device_gl::CommandBuffer>>,
             factory: gfx_device_gl::Factory,
             main_color: RenderTargetView<gfx_device_gl::Resources, ColorFormat>,
             main_depth: DepthStencilView<gfx_device_gl::Resources, DepthFormat>)
             -> System {
    System {
      renderer: OpenGlRenderer::new(factory, main_color, main_depth),
      encoder_recv: encoder_recv,
      encoder_send: encoder_send,
    }
  }
}

#[allow(unused_imports, unused_variables)]
impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    use specs::Join;
    use itertools::Itertools;

    let (mut window,
         camera_pos,
         own_entity,
         synchronized,
         entities,
         physical,
         disableds,
         renderables) = arg.fetch(|w| {
      (w.write_resource::<glutin::Window>(),
       w.read_resource::<CameraPos>(),
       w.write_resource::<Option<OwnEntity>>(),
       w.read::<SynchronizedAspect>(),
       w.entities(),
       w.read::<PhysicalAspect>(),
       w.read::<DisabledAspect>(),
       w.read::<RenderAspect>())
    });

    let mut encoder = self.encoder_recv.recv().unwrap();

    let &CameraPos(x, y, z) = camera_pos.deref();

    let camera_target = own_entity.clone()
      // Try to find our owned ent in the ent list
      .and_then(|ent| {
        (&entities, &synchronized)
          .iter()
          .filter(|&(_, synchro)| {
            let OwnEntity(ref own_ent) = ent;
            synchro == own_ent
          })
          .next()
      })
      .map(|(entity, _)| entity)
      .and_then(|true_ent| physical.get(true_ent))
      .map(|physical_aspect| physical_aspect.pos.clone());

    self.renderer.render_world(&mut encoder, window.deref_mut(), &(x, y, z), camera_target);

    (&physical, disableds.not(), &renderables)
      .iter()
      .foreach(|(physical_aspect, _, _)| {
        // TODO: use rendered_aspect to determine model
        self.renderer.render_model(&mut encoder, window.deref_mut(), physical_aspect, (x, y, z));
      });

    // Ship the encoder back to the main thread
    self.encoder_send.send(encoder).unwrap();
  }
}
