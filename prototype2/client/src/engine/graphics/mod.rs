use specs;
use glutin;
use gfx_device_gl;
use gfx;

use std::sync::mpsc::{Receiver, Sender};
use std::ops::Not;
use std::sync::RwLockReadGuard;

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

    let (mut window, camera_pos, own_entity, synchronized, entities, physical, disabled, render) =
      arg.fetch(|w| {
        (w.write_resource::<glutin::Window>(),
         w.read_resource::<CameraPos>(),
         w.write_resource::<Option<OwnEntity>>(),
         w.read::<SynchronizedAspect>(),
         w.entities(),
         w.read::<PhysicalAspect>(),
         w.read::<DisabledAspect>(),
         w.read::<RenderAspect>())
      });

    let camera_target = SelfRetriever::new(own_entity.clone(), &entities, &synchronized, &physical)
      .find_own_pos();

    // Retrieve encoder from main thread
    let mut encoder = self.encoder_recv.recv().unwrap();

    RenderWrapper::new(&mut self.renderer,
                       &mut encoder,
                       &mut window,
                       &camera_pos,
                       camera_target,
                       &render,
                       &physical,
                       &disabled)
      .render_frame();

    // Ship the encoder back to the main thread
    self.encoder_send.send(encoder).unwrap();
  }
}


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
      .map(|physical_aspect| physical_aspect.pos.clone())
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
  }
}
